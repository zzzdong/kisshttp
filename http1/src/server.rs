use std::{
    convert::Infallible,
    fmt::{self},
    future::Future,
    io::Cursor,
    pin::Pin,
};

use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter, ReadHalf, WriteHalf},
    select,
    sync::{mpsc, oneshot, watch},
};

use crate::{
    body::Body,
    error::{Error, ErrorKind},
    http::{ContentLength, RequestInfo},
};
use crate::{
    body::Sender,
    http::{Request, Response},
};

use crate::parser::{parse_request, ParseError, RawRequest};

const BUF_INIT_CAPACITY: usize = 4 * 1024 + 64;
const MAX_HEADER_SIZE: usize = 4 * 1024;

pub struct Pipeline {
    request_rx: mpsc::Receiver<Request>,
    response_tx: mpsc::Sender<(Response, oneshot::Sender<Result<(), Error>>)>,
}

impl Pipeline {
    fn new(
        request_rx: mpsc::Receiver<Request>,
        response_tx: mpsc::Sender<(Response, oneshot::Sender<Result<(), Error>>)>,
    ) -> Self {
        Pipeline {
            request_rx,
            response_tx,
        }
    }

    async fn next(&mut self) -> Option<Request> {
        self.request_rx.recv().await
    }

    async fn response(&mut self, resp: Response) -> Result<(), Error> {
        let (done_tx, done_rx) = oneshot::channel();

        self.response_tx.send((resp, done_tx)).await;

        done_rx.await;

        Ok(())
    }
}

struct Dispatcher<RW> {
    stream: RW,
    request_tx: mpsc::Sender<Request>,
    response_rx: mpsc::Receiver<(Response, oneshot::Sender<Result<(), Error>>)>,
}

impl<RW> Dispatcher<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(
        stream: RW,
        request_tx: mpsc::Sender<Request>,
        response_rx: mpsc::Receiver<(Response, oneshot::Sender<Result<(), Error>>)>,
    ) -> Self {
        Dispatcher {
            stream: stream,
            request_tx,
            response_rx,
        }
    }

    async fn dispatch(self) -> Result<(), Error> {
        let Dispatcher {
            stream,
            request_tx,
            response_rx,
        } = self;

        let (signal_tx, signal_rx) = mpsc::channel::<bool>(1);

        let (read_half, write_half) = tokio::io::split(stream);

        let reader = StreamReader::new(read_half, signal_tx, request_tx);

        let writer = StreamWriter::new(write_half, signal_rx, response_rx);

        let ret = tokio::join!(reader.run(), writer.run());

        Ok(())
    }

    async fn response(&mut self, resp: Response) -> Result<(), Error> {
        let buf = resp.header_buf();

        self.stream.write_all(&buf).await?;

        self.stream.flush().await?;

        Ok(())
    }
}

pub struct StreamReader<R> {
    stream: ReadHalf<R>,
    buffer: BytesMut,
    signal_tx: mpsc::Sender<bool>,
    request_tx: mpsc::Sender<Request>,
}

impl<R: AsyncRead> StreamReader<R> {
    fn new(
        stream: ReadHalf<R>,
        signal_tx: mpsc::Sender<bool>,
        request_tx: mpsc::Sender<Request>,
    ) -> Self {
        StreamReader {
            stream,
            signal_tx,
            request_tx,
            buffer: BytesMut::with_capacity(BUF_INIT_CAPACITY),
        }
    }

    async fn run(mut self) -> Result<(), Error> {
        let r_tx = self.request_tx.clone();

        loop {
            select! {
                ret = self.do_read() => {
                    match ret {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }

                _ = r_tx.closed() => {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn do_read(&mut self) -> Result<(), Error> {
        let mut info = RequestInfo::new();

        match self.read_request_header(&mut info).await {
            Ok(mut req) => {
                let (body, sender) = self.build_request_body(&info);

                req.body = body;

                // println!("=> {:?}", &req);

                self.request_tx.send(req).await.unwrap();

                self.read_request_body(&info, sender).await?;

                Ok(())
            }

            Err(err) => {
                self.signal_tx.send(true).await.unwrap();
                Err(err)
            }
        }
    }

    async fn read_request_header(&mut self, info: &mut RequestInfo) -> Result<Request, Error> {
        loop {
            let mut req = RawRequest::new();
            match parse_request(&self.buffer[..], &mut req) {
                Ok(parsed) => {
                    let ret = Request::from_raw_request(req, info);
                    self.buffer.advance(parsed);
                    return ret;
                }
                Err(ParseError::Incomplete) => {
                    if self.buffer.len() > MAX_HEADER_SIZE {
                        return Err(Error::new(ErrorKind::Protocol, ParseError::TooLarge));
                    }
                }
                Err(err) => {
                    return Err(Error::new(ErrorKind::Protocol, err));
                }
            }

            let n = self.stream.read_buf(&mut self.buffer).await?;
            if n == 0 {
                if self.buffer.len() > 8 * 1024 {
                    return Err(ParseError::TooLarge.into());
                }

                if self.buffer.capacity() == self.buffer.len() {
                    self.buffer.reserve(1024);
                    continue;
                }

                return Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, "").into());
            }
        }
    }

    fn build_request_body(&mut self, info: &RequestInfo) -> (Body, Option<Sender>) {
        fn new_body_channel() -> (Body, Option<Sender>) {
            let (s, b) = Body::channel();

            (b, Some(s))
        }

        match info.content_length {
            ContentLength::None => (Body::empty(), None),
            ContentLength::Close => new_body_channel(),
            ContentLength::Sized(len) => {
                if self.buffer.len() >= len {
                    let data = self.buffer.split_to(len);
                    return (Body::with_bytes(data), None);
                }

                new_body_channel()
            }
            ContentLength::Chunked => new_body_channel(),
        }
    }

    async fn read_request_body(
        &mut self,
        info: &RequestInfo,
        tx: Option<Sender>,
    ) -> Result<(), Error> {
        match info.content_length {
            ContentLength::None => Ok(()),
            ContentLength::Close => self.read_request_close_body(tx).await,
            ContentLength::Sized(len) => self.read_request_sized_body(len, tx).await,
            ContentLength::Chunked => self.read_request_chunked_body(tx).await,
        }
    }

    async fn read_request_sized_body(
        &mut self,
        len: usize,
        tx: Option<Sender>,
    ) -> Result<(), Error> {
        let mut need = len;

        loop {
            if self.buffer.len() >= need {
                let data = self.buffer.split_to(need).freeze();

                if let Some(ref tx) = tx {
                    tx.send(Ok(data)).await.unwrap();
                    return Ok(());
                }
            }

            let data = self.buffer.split().freeze();

            need -= data.len();

            if let Some(ref tx) = tx {
                tx.send(Ok(data)).await.unwrap();
            }

            // do read
            self.read_buf().await?;
        }
    }

    async fn read_request_close_body(&mut self, tx: Option<Sender>) -> Result<(), Error> {
        unimplemented!()
    }

    async fn read_request_chunked_body(&mut self, tx: Option<Sender>) -> Result<(), Error> {
        unimplemented!()
    }

    async fn read_buf(&mut self) -> Result<(), Error> {
        let n = self.stream.read_buf(&mut self.buffer).await?;
        if n == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, "").into());
        }

        Ok(())
    }
}

pub struct StreamWriter<W> {
    stream: WriteHalf<W>,
    signal_rx: mpsc::Receiver<bool>,
    response_rx: mpsc::Receiver<(Response, oneshot::Sender<Result<(), Error>>)>,
}

impl<W: AsyncWrite> StreamWriter<W> {
    fn new(
        stream: WriteHalf<W>,
        signal_rx: mpsc::Receiver<bool>,
        response_rx: mpsc::Receiver<(Response, oneshot::Sender<Result<(), Error>>)>,
    ) -> Self {
        StreamWriter {
            stream,
            signal_rx,
            response_rx,
        }
    }

    async fn run(mut self) -> Result<(), Error> {
        loop {
            select! {
                _ = self.signal_rx.recv() => {
                    return Ok(());
                }

                ret = self.response_rx.recv() => {
                    match ret {
                        Some((resp, tx)) => {
                            let ret = self.write_response(resp).await;

                            tx.send(ret);
                        }
                        None => {
                            return Ok(());
                        }
                    }

                }
            }
        }

        Ok(())
    }

    async fn write_response(&mut self, resp: Response) -> Result<(), Error> {
        let data = resp.header_buf();
        self.stream.write_all(&data).await;
        self.stream.flush().await;

        Ok(())
    }
}

pub async fn serve<IO>(io: IO, handler: impl Handler + Send + 'static) -> Result<(), Error>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    let mut handler = handler;

    let (request_tx, request_rx) = mpsc::channel(1);
    let (response_tx, response_rx) = mpsc::channel(1);

    let mut pipeline = Pipeline::new(request_rx, response_tx);

    let dispatcher = Dispatcher::new(io, request_tx, response_rx);

    tokio::spawn(async move {
        loop {
            match pipeline.next().await {
                Some(req) => {
                    let resp = handler.call(req).await;
                    pipeline.response(resp).await.unwrap();
                }
                None => {
                    break;
                }
            }
        }
    });

    dispatcher.dispatch().await.unwrap();

    Ok(())
}

#[async_trait::async_trait]
pub trait Handler {
    async fn call(&mut self, req: Request) -> Response;
}

#[async_trait::async_trait]
impl<Fut, F: Send + Sync + 'static> Handler for F
where
    F: FnMut(Request) -> Fut,
    Fut: Future<Output = Response> + Send + Sync + 'static,
{
    async fn call(&mut self, req: Request) -> Response {
        self(req).await
    }
}

#[cfg(test)]
mod test {
    use tokio::net::TcpListener;

    use crate::http::{Request, Response};

    use super::serve;

    #[tokio::test]
    async fn test_serve() {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

        loop {
            let connection = listener.accept().await.unwrap();

            tokio::spawn(async move {
                serve(connection.0, |req: Request| {
                    Box::pin(async move { Response::new() })
                })
                .await
            });
        }
    }
}

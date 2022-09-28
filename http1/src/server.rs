use std::{
    convert::Infallible,
    fmt::{self},
    future::Future,
    io::Cursor,
    pin::Pin,
};

use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

use crate::error::{Error, ErrorKind};
use crate::http::{Request, Response};

use crate::parser::{parse_request, ParseError, RawRequest};

const BUF_INIT_CAPACITY: usize = 4 * 1024 + 64;
const MAX_HEADER_SIZE: usize = 4 * 1024;

pub struct Pipeline<RW> {
    stream: BufWriter<RW>,
    buffer: BytesMut,
}

impl<RW> Pipeline<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: RW) -> Self {
        Pipeline {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(BUF_INIT_CAPACITY),
        }
    }

    // pub async fn next(&mut self) -> Result<(), Error> {
    //     let req = self.read_request().await?;

    //     Ok(())
    // }

    async fn read_request(&mut self) -> Result<Request, Error> {
        loop {
            let mut req = RawRequest::new();
            match parse_request(&self.buffer[..], &mut req) {
                Ok(parsed) => {
                    let ret = Request::try_from(req);
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
        }
    }

    async fn response(&mut self, resp: Response) -> Result<(), Error> {
        self.stream
            .write_all(
                b"HTTP/1.1 200 Ok\r\nContent-Length: 5\r\nConnection: keep-alive\r\n\r\nHello",
            )
            .await
            .map_err(Into::into)
    }
}

pub async fn serve<IO>(io: IO, handler: impl Handler) -> Result<(), Error>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    let mut handler = handler;
    let mut pipeline = Pipeline::new(io);

    loop {
        let req = pipeline.read_request().await.unwrap();
        let resp = handler.call(req).await;
        pipeline.response(resp).await?;
    }
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

    use super::{serve};

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

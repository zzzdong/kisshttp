use std::{
    convert::Infallible,
    fmt::{self},
    future::Future,
    pin::Pin,
};

use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tower::{Service, ServiceExt};

use crate::{
    http::{Request, Response},
    parser::{parse_request, ParseError, RawRequest},
};

const BUF_INIT_CAPACITY: usize = 4 * 1024 + 64;
const MAX_HEADER_SIZE: usize = 4 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Protocol,
}

pub struct Error {
    kind: ErrorKind,
    cause: Box<dyn std::error::Error + Send + 'static>,
}

impl Error {
    pub fn new(kind: ErrorKind, cause: impl std::error::Error + Send + 'static) -> Self {
        Error {
            kind,
            cause: Box::new(cause),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("cause", &self.cause)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("cause", &self.cause)
            .finish()
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::Io, err)
    }
}

impl std::error::Error for Error {}

pub struct BufferedIO<RW> {
    io: RW,
    read_buf: BytesMut,
}

impl<RW> BufferedIO<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(io: RW) -> Self {
        BufferedIO {
            io,
            read_buf: BytesMut::with_capacity(BUF_INIT_CAPACITY),
        }
    }

    pub async fn do_read(&mut self) -> Result<usize, Error> {
        self.io
            .read_buf(&mut self.read_buf)
            .await
            .map_err(Into::into)
    }

    pub fn bufferd(&self) -> &[u8] {
        &self.read_buf
    }

    pub fn bufferd_len(&self) -> usize {
        self.read_buf.len()
    }

    pub fn advance(&mut self, len: usize) {
        self.read_buf.advance(len);
    }

    pub async fn do_write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.io.write_all(buf).await?;
        Ok(())
    }
}

pub struct Pipeline<RW> {
    io: BufferedIO<RW>,
}

impl<RW> Pipeline<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(io: RW) -> Self {
        Pipeline {
            io: BufferedIO::new(io),
        }
    }

    // pub async fn next(&mut self) -> Result<(), Error> {
    //     let req = self.read_request().await?;

    //     Ok(())
    // }

    async fn read_request(&mut self) -> Result<Request, Error> {
        loop {
            let read_len = self.io.do_read().await?;
            if read_len > 0 {
                let mut req = RawRequest::new();
                match parse_request(self.io.bufferd(), &mut req) {
                    Ok(parsed) => {
                        let ret = Request::from(req);
                        self.io.advance(parsed);
                        return Ok(ret);
                    }
                    Err(ParseError::Incomplete) => {
                        if self.io.bufferd_len() > MAX_HEADER_SIZE {
                            return Err(Error::new(ErrorKind::Protocol, ParseError::TooLarge));
                        }
                    }
                    Err(err) => {
                        return Err(Error::new(ErrorKind::Protocol, err));
                    }
                }
            }
        }
    }

    async fn response(&mut self, resp: Response) -> Result<(), Error> {
        self.io
            .do_write(b"HTTP/1.1 200 Ok\r\nContent-Length: 5\r\nConnection: keep-alive\r\n\r\nHello")
            .await
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
        match handler.call(req).await {
            Ok(resp) => {
                pipeline.response(resp).await;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

#[async_trait::async_trait]
pub trait Handler {
    type Error: std::error::Error;

    async fn call(&mut self, req: Request) -> Result<Response, Self::Error>;
}

#[async_trait::async_trait]
impl<E, Fut, F: Send + Sync + 'static> Handler for F
where
    E: std::error::Error + 'static,
    F: FnMut(Request) -> Fut,
    Fut: Future<Output = Result<Response, E>> + Send + Sync + 'static,
{
    type Error = E;

    async fn call(&mut self, req: Request) -> Result<Response, Self::Error> {
        self(req).await
    }
}

#[cfg(test)]
mod test {
    use tokio::net::TcpListener;

    use crate::http::{Request, Response};

    use super::{serve, Error};

    #[tokio::test]
    async fn test_serve() {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

        loop {
            let connection = listener.accept().await.unwrap();

            tokio::spawn(async move {
                serve(connection.0, |req: Request| {
                    Box::pin(async move {
                        // println!("{:?}", req);
                        Ok::<Response, Error>(Response::new())
                    })
                })
                .await
            });
        }
    }
}

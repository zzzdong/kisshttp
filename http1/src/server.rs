use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::{parser::{ParseError, RawRequest, parse_request}};

const BUF_INIT_CAPACITY: usize = 4 * 1024;

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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::Io, err)
    }
}

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

    pub fn bufferd(&self) -> BytesMut {
        self.read_buf.clone()
    }

    pub fn bufferd_len(&self) -> usize {
        self.read_buf.len()
    }

    pub fn advance(&mut self, len: usize) {
        self.read_buf.advance(len);
    }
}

pub struct Pipeline<RW> {
    io: BufferedIO<RW>,
}

impl<RW> Pipeline<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    // pub async fn next(&mut self) -> Result<(), Error> {
    //     let req = self.read_request().await?;



    //     Ok(())
    // }

    // async fn read_request<'a>(&'a mut self) -> Result<RawRequest<'a>, Error> {
    //     loop {
    //         let read_len = self.io.do_read().await?;
    //         if read_len > 0 {
    //             let mut req = RawRequest::new();
    //             match parse_request(&) {
    //                 Ok( parsed) => {
    //                     self.io.advance(parsed);
    //                     return Ok(req);
    //                 }
    //                 Err(ParseError::Incomplete) => {}
    //                 Err(err) => {}
    //             }
    //         }
    //     }
    // }
}

pub struct OneRound {}

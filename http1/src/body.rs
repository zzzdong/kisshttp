use bytes::{Bytes, BytesMut};
use tokio::sync::mpsc;

use crate::error::Error;

#[derive(Debug)]
enum Kind {
    Empty,
    Once(Bytes),
    Channel(mpsc::Receiver<Result<Bytes, Error>>),
}

#[derive(Debug)]
pub struct Body {
    kind: Kind,
}

impl Body {
    fn new(kind: Kind) -> Self {
        Body { kind }
    }

    pub fn empty() -> Self {
        Body::new(Kind::Empty)
    }

    pub fn with_bytes(b: impl Into<Bytes>) -> Self {
        Body::new(Kind::Once(b.into()))
    }

    pub fn channel() -> (Sender, Body) {
        let (tx, rx) = mpsc::channel(1);

        (Sender::new(tx), Body::new(Kind::Channel(rx)))
    }

    pub async fn data(&mut self) -> Result<Option<Bytes>, Error> {
        match &mut self.kind {
            Kind::Empty => Ok(None),
            Kind::Once(d) => {
                let data = d.clone();
                self.kind = Kind::Empty;
                Ok(Some(data))
            }
            Kind::Channel(rx) => match rx.recv().await {
                Some(Ok(d)) => Ok(Some(d)),
                Some(Err(err)) => Err(err),
                None => Ok(None),
            },
        }
    }
}

pub struct Sender {
    tx: mpsc::Sender<Result<Bytes, Error>>,
}

impl Sender {
    fn new(tx: mpsc::Sender<Result<Bytes, Error>>) -> Self {
        Sender { tx }
    }

    pub async fn send(&self, data: Result<Bytes, Error>) -> Result<(), Error> {
        self.tx
            .send(data)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "").into())
    }

    pub async fn closed(&self) {
        self.tx.closed().await
    }
}

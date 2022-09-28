use http1::{
    http::{Request, Response},
    server::serve,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (stream, _remote_addr) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            serve(stream, |req: Request| {
                Box::pin(async move {
                    // println!("{:?}", req);
                    let mut resp = Response::new();
                    resp.header_map.append(b"Content-Length", b"0");
                    resp.header_map.append(b"Connection", b"keep-alive");
                    resp
                })
            })
            .await
        });
    }
}

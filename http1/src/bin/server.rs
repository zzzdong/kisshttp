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
            serve(stream, |mut req: Request| {
                Box::pin(async move {
                    // println!("{:?}", req);
                    // while let Ok(Some(d)) = req.body.data().await {
                    //     println!("{:?}", String::from_utf8_lossy(&d));
                    // }

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

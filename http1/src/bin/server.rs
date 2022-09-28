use http1::{
    http::{Request, Response},
    server::serve,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let connection = listener.accept().await.unwrap();

        tokio::spawn(async move {
            serve(connection.0, |req: Request| {
                Box::pin(async move {
                    // println!("{:?}", req);
                    Response::new()
                })
            })
            .await
        });
    }
}

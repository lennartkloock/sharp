use std::error::Error;
use std::net::SocketAddr;
use hyper::service::service_fn;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let in_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
    let out_addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

    let out_addr_clone = out_addr.clone();

    let listener = TcpListener::bind(in_addr).await?;

    println!("Listening on http://{}", in_addr);
    println!("Proxying on http://{}", out_addr);

    loop {
        let (stream, _) = listener.accept().await?;

        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        let service = service_fn(move |mut req| {
            let uri_string = format!(
                "http://{}{}",
                out_addr_clone,
                req.uri()
                    .path_and_query()
                    .map(|x| x.as_str())
                    .unwrap_or("/")
            );
            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;

            let host = req.uri().host().expect("uri has no host");
            let port = req.uri().port_u16().unwrap_or(80);
            let addr = format!("{}:{}", host, port);

            async move {
                let client_stream = TcpStream::connect(addr).await.unwrap();

                let (mut sender, conn) = hyper::client::conn::handshake(client_stream).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        println!("Connection failed: {:?}", err);
                    }
                });

                sender.send_request(req).await
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::Http::new()
                .serve_connection(stream, service)
                .await
            {
                println!("Failed to serve the connection: {:?}", err);
            }
        });
    }
}

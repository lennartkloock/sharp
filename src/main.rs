use std::{error::Error, net::SocketAddr};

use hyper::{service::service_fn, Body, Response};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    info!("CI/CD Test");
    info!("{VERSION_STRING}");

    let in_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
    let out_addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

    let out_addr_clone = out_addr.clone();

    let listener = TcpListener::bind(in_addr).await?;

    info!("Listening on http://{}", in_addr);
    info!("Proxying on http://{}", out_addr);

    loop {
        let (stream, client_addr) = listener.accept().await?;

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
            req.headers_mut().insert(
                "X-Sharp-Client-Ip",
                client_addr.ip().to_string().parse().unwrap(),
            );
            req.headers_mut().insert(
                "X-Sharp-Client-Port",
                client_addr.port().to_string().parse().unwrap(),
            );

            let host = req.uri().host().expect("uri has no host");
            let port = req.uri().port_u16().unwrap_or(80);
            let addr = format!("{}:{}", host, port);

            async move {
                match TcpStream::connect(addr).await {
                    Ok(client_stream) => {
                        let (mut sender, conn) =
                            hyper::client::conn::handshake(client_stream).await?;
                        tokio::task::spawn(async move {
                            if let Err(err) = conn.await {
                                println!("Connection failed: {:?}", err);
                            }
                        });

                        sender.send_request(req).await
                    }
                    Err(e) => Ok(Response::builder()
                        .status(502)
                        .body(Body::from(format!(
                            "failed to connect to upstream server\n{}\n\n{VERSION_STRING}",
                            e
                        )))
                        .unwrap()),
                }
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::Http::new()
                .http1_preserve_header_case(true)
                .http1_title_case_headers(true)
                .serve_connection(stream, service)
                .await
            {
                error!("failed to serve the connection: {:?}", err);
            }
        });
    }
}

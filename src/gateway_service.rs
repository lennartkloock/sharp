use std::net::SocketAddr;
use hyper::{Body, Request, Response};
use tokio::net::TcpStream;

pub async fn service(mut req: Request<Body>, out_addr: SocketAddr) -> hyper::Result<Response<Body>> {
    let uri_string = format!(
        "http://{}{}",
        out_addr,
        req.uri()
            .path_and_query()
            .map(|x| x.as_str())
            .unwrap_or("/")
    );
    let uri = uri_string.parse().unwrap();
    *req.uri_mut() = uri;
    // req.headers_mut().insert(
    //     "X-Sharp-Client-Ip",
    //     client_addr.ip().to_string().parse().unwrap(),
    // );
    // req.headers_mut().insert(
    //     "X-Sharp-Client-Port",
    //     client_addr.port().to_string().parse().unwrap(),
    // );

    let host = req.uri().host().expect("uri has no host");
    let port = req.uri().port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

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
                "failed to connect to upstream server\n{}\n\n{}",
                e,
                crate::VERSION_STRING
            )))
            .unwrap()),
    }
}

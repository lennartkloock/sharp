use hyper::http::Request;
use tracing::info;

pub fn is_exception<T>(req: &Request<T>) -> bool {
    let exception = ["/favicon.ico", "/robots.txt", "/sitemap.xml"].contains(&req.uri().path());
    if exception {
        info!("`{}` is an exception", req.uri());
    }
    exception
}

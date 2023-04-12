use hyper::http::Request;

pub fn is_exception<T>(req: &Request<T>) -> bool {
    ["/favicon.ico", "/robots.txt", "/sitemap.xml"].contains(&req.uri().path())
}

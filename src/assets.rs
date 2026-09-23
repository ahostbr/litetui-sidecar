use std::borrow::Cow;

use percent_encoding::percent_decode_str;
use wry::http::{header, Request, Response, StatusCode};

const INDEX: &[u8] = include_bytes!("../index.html");
pub fn response(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let decoded = percent_decode_str(request.uri().path()).decode_utf8_lossy();
    let path = decoded.trim_start_matches('/');
    let (bytes, mime): (&'static [u8], &'static str) = match path {
        "" | "index.html" => (INDEX, "text/html; charset=utf-8"),
        _ => return error_response(StatusCode::NOT_FOUND, b"Not found"),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header("X-Content-Type-Options", "nosniff")
        .header("Cache-Control", "no-store")
        .header(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self'; connect-src 'self' blob:; object-src 'none'; base-uri 'none'; frame-src 'none'",
        )
        .body(Cow::Borrowed(bytes))
        .expect("static asset response")
}

fn error_response(status: StatusCode, body: &'static [u8]) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("X-Content-Type-Options", "nosniff")
        .body(Cow::Borrowed(body))
        .expect("static error response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_only_offline_shell_without_privileged_web_api() {
        let request = Request::builder()
            .uri("sidecar://localhost/index.html")
            .body(Vec::new())
            .unwrap();
        let response = response(request);
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .body()
            .windows(b"LiteTUI".len())
            .any(|chunk| chunk == b"LiteTUI"));
        let policy = response
            .headers()
            .get("Content-Security-Policy")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(policy.contains("frame-src 'none'"));
        assert!(policy.contains("connect-src 'self'"));
        let request = Request::builder()
            .uri("sidecar://localhost/../../Cargo.toml")
            .body(Vec::new())
            .unwrap();
        assert_eq!(super::response(request).status(), StatusCode::NOT_FOUND);
    }
}

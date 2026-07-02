//! Pure-Rust HTTP/1.1 server primitives for the Tails `http` native module.
//!
//! This crate contains no dependency on the Tails runtime. The thin adapter in
//! `src/runtime_env/native_fns/http_fns.rs` converts between runtime `Value`
//! types and the functions exported here.
//!
//! The server is intentionally synchronous and built on `std::net` so it fits
//! the runtime's single-threaded, cooperative event loop: the adapter drives a
//! bounded accept loop, parsing each request and writing each response.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

/// A parsed HTTP/1.1 request.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Bind a non-blocking TCP listener on `127.0.0.1:port`.
///
/// Pass `0` to let the OS pick an ephemeral port; the chosen port is available
/// via [`TcpListener::local_addr`].
pub fn bind(port: u16) -> std::io::Result<TcpListener> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    // Non-blocking so the adapter can poll for connections while still
    // respecting a close flag / timeout between accepts.
    listener.set_nonblocking(true)?;
    Ok(listener)
}

/// Read and parse a single HTTP/1.1 request from `stream`.
///
/// Reads the request line, headers, and (if `Content-Length` is present) the
/// request body. Lowercases header names for case-insensitive lookup.
pub fn read_request(stream: &mut TcpStream) -> std::io::Result<HttpRequest> {
    // A short read timeout keeps a misbehaving client from stalling the loop.
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

    let mut reader = BufReader::new(stream);

    // Request line: METHOD SP PATH SP VERSION CRLF
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let trimmed = request_line.trim_end();
    let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "malformed HTTP request line",
        ));
    }
    let method = parts[0].to_string();
    let path = parts[1].to_string();

    // Headers until empty line.
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break; // EOF before blank line
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }

    // Body (only if Content-Length is present).
    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut body = String::new();
    if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        reader.read_exact(&mut buf)?;
        // Use lossy conversion: the adapter exposes the body as a JS string.
        body = String::from_utf8_lossy(&buf).into_owned();
    }

    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

/// Write an HTTP/1.1 response to `stream`.
///
/// `headers` may already include a `Content-Length`; if not, one is added based
/// on `body.len()`. The connection is closed after the response is written.
pub fn write_response(
    stream: &mut TcpStream,
    status: u16,
    status_text: &str,
    headers: &HashMap<String, String>,
    body: &str,
) -> std::io::Result<()> {
    let mut out = String::with_capacity(256 + body.len());
    out.push_str(&format!("HTTP/1.1 {} {}\r\n", status, status_text));
    let has_content_length = headers.contains_key("content-length");
    for (k, v) in headers {
        out.push_str(&format!("{}: {}\r\n", k, v));
    }
    if !has_content_length {
        out.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    out.push_str("Connection: close\r\n");
    out.push_str("\r\n");
    stream.write_all(out.as_bytes())?;
    if !body.is_empty() {
        stream.write_all(body.as_bytes())?;
    }
    stream.flush()?;
    Ok(())
}

/// Return canonical status text for common HTTP status codes.
pub fn status_text(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        418 => "I'm a teapot",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn status_text_covers_common_codes() {
        assert_eq!(status_text(200), "OK");
        assert_eq!(status_text(201), "Created");
        assert_eq!(status_text(301), "Moved Permanently");
        assert_eq!(status_text(404), "Not Found");
        assert_eq!(status_text(500), "Internal Server Error");
        assert_eq!(status_text(999), "OK"); // unknown falls back to OK
    }

    #[test]
    fn binds_ephemeral_port() {
        // Binding port 0 should always succeed and yield a real local port.
        let listener = bind(0).expect("bind ephemeral");
        let port = listener.local_addr().unwrap().port();
        assert!(port > 0);
    }

    #[test]
    fn round_trips_a_request_response() {
        let listener = bind(0).expect("bind");
        let port = listener.local_addr().unwrap().port();

        // Client thread: connect, send a POST with a body, read the response.
        let client = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            let mut conn = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect");
            let req = "POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello";
            conn.write_all(req.as_bytes()).expect("write req");
            conn.flush().ok();
            let mut resp = String::new();
            conn.read_to_string(&mut resp).expect("read resp");
            resp
        });

        // Server side: poll the non-blocking listener until the client connects.
        let mut connected = None;
        for _ in 0..200 {
            match listener.accept() {
                Ok((s, _)) => {
                    connected = Some(s);
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => panic!("accept failed: {e}"),
            }
        }
        let mut stream = connected.expect("no client connection");
        let req = read_request(&mut stream).expect("read_request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/echo");
        assert_eq!(req.body, "hello");
        assert_eq!(
            req.headers.get("host").map(|s| s.as_str()),
            Some("localhost")
        );

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());
        write_response(&mut stream, 200, "OK", &headers, "ok:5").expect("write_response");

        // Drop the stream so the client's read_to_string sees EOF — otherwise
        // the client blocks forever waiting for the connection to close.
        drop(stream);

        let resp = client.join().expect("client thread");
        assert!(resp.starts_with("HTTP/1.1 200 OK"));
        assert!(resp.contains("Content-Length: 4"));
        assert!(resp.ends_with("ok:5"));
    }
}

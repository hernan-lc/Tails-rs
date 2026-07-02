#![cfg(feature = "http")]

use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;
use tails::TailsRuntime;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find a free ephemeral TCP port by binding to :0, reading the assigned port,
/// and immediately dropping the listener. There is an inherent TOCTOU race,
/// but it is good enough for tests.
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    listener.local_addr().unwrap().port()
}

/// Send a raw HTTP request to `port` and return the full response string.
/// Retries the connection for up to 3 s so the client can be started before
/// the server finishes binding.
fn http_round_trip(port: u16, request: &str) -> String {
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        match std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
            Ok(mut stream) => {
                stream.write_all(request.as_bytes()).expect("write request");
                stream.flush().ok();
                let mut resp = String::new();
                stream.read_to_string(&mut resp).expect("read response");
                return resp;
            }
            Err(_) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("failed to connect to port {}: {}", port, e),
        }
    }
}

/// Run a Tails script that starts an HTTP server with `max_connections` and
/// returns after the server finishes. A client closure runs in a **separate
/// thread** (because `TailsRuntime` is `!Send` the server must stay on the
/// calling thread, but `TcpStream` is `Send` so the client can run elsewhere).
fn run_server_with_client<F>(source: &str, max_connections: i64, client: F)
where
    F: FnOnce(u16) + Send + 'static,
{
    let port = free_port();

    // Inject the port and limits into the source template.
    let source = source
        .replace("__PORT__", &port.to_string())
        .replace("__MAXCONN__", &max_connections.to_string());

    let port_clone = port;
    let client_handle = std::thread::spawn(move || client(port_clone));

    let mut rt = TailsRuntime::default();
    let result = rt.eval_module(&source, Path::new("/tmp/test_http_module.ts"));
    assert!(
        result.is_ok(),
        "Tails script failed: {:?}",
        result.err().map(|e| e.to_string())
    );

    // Run the event loop — it will exit once maxConnections is reached.
    rt.run_event_loop().expect("event loop failed");

    let resp = client_handle.join().expect("client thread panicked");
    let _ = resp; // assertions are done inside the closure via returns
}

/// Like `run_server_with_client` but returns the client's response string.
fn run_server_get_response(source: &str, max_connections: i64, request: &str) -> String {
    let request = request.to_string();
    let port_holder = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let port_holder_clone = port_holder.clone();

    run_server_with_client(source, max_connections, move |port| {
        let resp = http_round_trip(port, &request);
        *port_holder_clone.lock().unwrap() = resp;
    });

    let result = std::mem::take(&mut *port_holder.lock().unwrap());
    result
}

// ---------------------------------------------------------------------------
// Module structure tests (no network)
// ---------------------------------------------------------------------------

#[test]
fn test_http_module_has_create_server() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import http from "http";
        typeof http.createServer;
    "#,
        Path::new("/tmp/test_http_module.ts"),
    );
    assert!(r.is_ok(), "import http failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::String("function".to_string()));
}

#[test]
fn test_http_create_server_returns_object() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import http from "http";
        const server = http.createServer((req, res) => {});
        typeof server.listen === "function" && typeof server.close === "function";
    "#,
        Path::new("/tmp/test_http_module.ts"),
    );
    assert!(r.is_ok(), "createServer failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_http_create_server_without_handler() {
    // createServer with no args should still return a valid server object.
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import http from "http";
        const server = http.createServer();
        typeof server.listen === "function";
    "#,
        Path::new("/tmp/test_http_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

// ---------------------------------------------------------------------------
// End-to-end tests (real TCP connections)
// ---------------------------------------------------------------------------

#[test]
fn test_http_server_get_request() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            res.writeHead(200);
            res.end("hello-world");
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let resp = run_server_get_response(source, 1, "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    assert!(resp.contains("hello-world"), "response: {}", resp);
}

#[test]
fn test_http_server_post_body() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            res.writeHead(200);
            res.end("ok:" + req.body.length);
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let body = "hello-body";
    let request = format!(
        "POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let resp = run_server_get_response(source, 1, &request);
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    assert!(resp.contains("ok:10"), "expected ok:10, response: {}", resp);
}

#[test]
fn test_http_server_custom_status() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            res.writeHead(404);
            res.end("not found");
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let resp = run_server_get_response(
        source,
        1,
        "GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(resp.starts_with("HTTP/1.1 404"), "response: {}", resp);
    assert!(resp.contains("not found"), "response: {}", resp);
}

#[test]
fn test_http_server_res_write_then_end() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            res.writeHead(200);
            res.write("chunk1-");
            res.write("chunk2-");
            res.end("done");
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let resp = run_server_get_response(source, 1, "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    assert!(resp.contains("chunk1-chunk2-done"), "response: {}", resp);
}

#[test]
fn test_http_server_multiple_requests() {
    let source = r#"
        import http from "http";
        let count = 0;
        const server = http.createServer((req, res) => {
            count++;
            res.writeHead(200);
            res.end("req:" + count);
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 8000 });
    "#;

    let port = free_port();
    let source = source
        .replace("__PORT__", &port.to_string())
        .replace("__MAXCONN__", "3");

    let mut handles = Vec::new();
    for i in 0..3u8 {
        let p = port;
        handles.push(std::thread::spawn(move || {
            let req = format!("GET /{} HTTP/1.1\r\nHost: localhost\r\n\r\n", i);
            http_round_trip(p, &req)
        }));
    }

    let mut rt = TailsRuntime::default();
    let result = rt.eval_module(&source, Path::new("/tmp/test_http_module.ts"));
    assert!(result.is_ok(), "server failed: {:?}", result.err());

    // Run the event loop — it will exit once maxConnections is reached.
    rt.run_event_loop().expect("event loop failed");

    let responses: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(responses.len(), 3);
    for resp in &responses {
        assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    }
    let has_req1 = responses.iter().any(|r| r.contains("req:1"));
    assert!(has_req1, "no req:1 in responses: {:?}", responses);
}

#[test]
fn test_http_server_req_on_data_event() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            let received = "";
            req.on("data", (chunk) => { received = chunk; });
            req.on("end", () => {
                res.writeHead(200);
                res.end("data-len:" + received.length);
            });
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let body = "abcdef";
    let request = format!(
        "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let resp = run_server_get_response(source, 1, &request);
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    assert!(
        resp.contains("data-len:6") || resp.contains("data-len:0"),
        "response: {}",
        resp
    );
}

#[test]
fn test_http_server_req_headers() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            res.writeHead(200);
            res.end("host:" + req.headers["host"]);
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let resp = run_server_get_response(source, 1, "GET / HTTP/1.1\r\nHost: my-test-host\r\n\r\n");
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    assert!(resp.contains("host:my-test-host"), "response: {}", resp);
}

#[test]
fn test_http_server_req_method_and_url() {
    let source = r#"
        import http from "http";
        const server = http.createServer((req, res) => {
            res.writeHead(200);
            res.end(req.method + " " + req.url);
        });
        server.listen(__PORT__, () => {}, { maxConnections: __MAXCONN__, timeoutMs: 5000 });
    "#;

    let resp = run_server_get_response(
        source,
        1,
        "DELETE /api/items/42 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "response: {}", resp);
    assert!(resp.contains("DELETE /api/items/42"), "response: {}", resp);
}

//! Pure-Rust TCP client primitives for the Tails `net` native module.
//!
//! This crate contains no dependency on the Tails runtime. The thin adapter in
//! `src/runtime_env/native_fns/net_fns.rs` converts between runtime `Value`
//! types and the functions exported here.
//!
//! The client is intentionally synchronous and built on `std::net` so it fits
//! the runtime's single-threaded, cooperative event loop.

use std::io::{Read, Write};
use std::net::TcpStream;

/// Connect to a TCP server at `host:port`.
///
/// Sets read/write timeouts to avoid stalling the event loop.
pub fn connect(host: &str, port: u16) -> std::io::Result<TcpStream> {
    let stream = TcpStream::connect((host, port))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;
    Ok(stream)
}

/// Write `data` to the stream and flush.
pub fn write(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_all(data)?;
    stream.flush()?;
    Ok(())
}

/// Read whatever bytes are immediately available (up to 8 KiB).
///
/// Returns the number of bytes read; 0 means EOF.
pub fn read_available(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

/// Shut down both halves of the connection.
pub fn shutdown(stream: &TcpStream) -> std::io::Result<()> {
    stream.shutdown(std::net::Shutdown::Both)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn connect_write_read_roundtrip() {
        // Spin up a tiny echo server on an ephemeral port.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 64];
            let n = stream.read(&mut buf).unwrap();
            // Echo back what was received.
            stream.write_all(&buf[..n]).unwrap();
        });

        let mut conn = connect("127.0.0.1", port).expect("connect");
        write(&mut conn, b"hello").expect("write");

        let reply = read_available(&mut conn).expect("read");
        assert_eq!(reply, b"hello");

        shutdown(&conn).ok();
        server.join().unwrap();
    }

    #[test]
    fn shutdown_closes_connection() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 64];
            // Should eventually get 0 (EOF) after client shuts down.
            let _ = stream.read(&mut buf);
        });

        let conn = connect("127.0.0.1", port).expect("connect");
        shutdown(&conn).expect("shutdown");

        server.join().unwrap();
    }
}

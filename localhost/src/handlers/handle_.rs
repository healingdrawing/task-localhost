use http::Request;
use mio::net::TcpStream;
use std::io:: Write;

/// just for test
pub fn handle_request(request: Request<Vec<u8>>, stream: &mut TcpStream) {
  // For simplicity, just send a "Hello, World!" response
  let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
  stream.write_all(response.as_bytes()).unwrap();
  stream.flush().unwrap();
}
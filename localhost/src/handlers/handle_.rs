use http::{Request, Response, StatusCode};
use mio::net::TcpStream;
use std::io:: Write;
use std::path::PathBuf;

use crate::server::core::ServerConfig;
use crate::handlers::handle_cgi::handle_cgi;
use crate::handlers::handle_all::handle_all;
use crate::stream::write_::write_response_into_stream;

/// just for test
pub fn handle_request(
  request: &Request<Vec<u8>>,
  zero_path_buf: PathBuf,
  server_config: ServerConfig
) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>>{
  
  // todo!("handle_request: implement the logic. but first refactor to handle unwrap() more safe. to prevent panics");
  
  // try to manage the cgi request case strictly and separately,
  // to decrease vulnerability, because cgi is old, unsafe and not recommended to use.
  // Also, the task is low quality, because audit question ask only to check
  // the cgi with chunked and unchunked requests, so method check is not implemented,
  // because according to HTTP/1.1 standard, a not POST method can have body too
  let path = request.uri().path();
  let parts: Vec<&str> = path.split('/').collect();
  
  let response = match parts.as_slice(){
    ["", "cgi", "useless.py", file_path @ ..] => {
      handle_cgi(
        zero_path_buf,
        "useless.py".to_string(),
        file_path.join(&std::path::MAIN_SEPARATOR.to_string()),
        request,
        server_config,
      )
    },
    ["", "uploads", file_path @ ..] => {
      //todo :implement the response for uploads case. GET, POST, DELETE
      dummy_200_response()
    },
    _ => {
      // todo : implement the response for other cases
      handle_all( zero_path_buf, request, server_config, )
      // dummy_200_response()
    }
  };
  
  // match write_response_into_stream(stream, response){
  //   Ok(_) => println!("Response sent"),
  //   Err(e) => eprintln!("Failed to send response: {}", e),
  // }
  
  // match stream.flush(){
  //   Ok(_) => println!("Response flushed"),
  //   Err(e) => eprintln!("Failed to flush response: {}", e),
  // };
  
  // match stream.shutdown(std::net::Shutdown::Both) {
  //   Ok(_) => println!("Connection closed successfully"),
  //   Err(e) => eprintln!("Failed to close connection: {}", e),
  // }
  
  Ok(response)
}

/// todo: remove dev gap
fn dummy_200_response() -> Response<Vec<u8>>{
  let body = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World! dummy_200_response\n\n";
  Response::builder()
  .status(StatusCode::OK)
  .body(body.as_bytes().to_vec())
  .unwrap()
}
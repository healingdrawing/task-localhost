use std::time::{Instant, Duration};
use std::io::{self};

use futures::AsyncReadExt;
use async_std::net::TcpStream;

use crate::debug::append_to_file;
use crate::server::core::ServerConfig;
use crate::server::find::server_config_from_headers_buffer_or_use_default;
use crate::stream::errors::{ERROR_400_HEADERS_READ_TIMEOUT, ERROR_400_HEADERS_READING_STREAM, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_500_INTERNAL_SERVER_ERROR};
use crate::stream::read_chunked::read_chunked;
use crate::stream::read_unchunked::read_unchunked;

/// Read from the stream until timeout or EOF
/// 
/// returns a tuple of two vectors: (headers_buffer, body_buffer)
pub async fn read_with_timeout(
  timeout: Duration,
  stream: &mut TcpStream,
  headers_buffer: &mut Vec<u8>,
  body_buffer: &mut Vec<u8>,
  server_configs: &Vec<ServerConfig>,
  global_error_string: &mut String,
) -> ServerConfig {

  append_to_file("\nINSIDE read_with_timeout").await;
  
  // Start the timer
  let start_time = Instant::now();
  
  // Read from the stream until timeout or EOF
  let mut buf = [0; 1];
  
  // ------------------------------------
  // collect request headers section
  // ------------------------------------
  
  loop {
    
    // Check if the timeout has expired
    if start_time.elapsed() >= timeout {
      eprintln!("ERROR: Headers read timed out");
      *global_error_string = ERROR_400_HEADERS_READ_TIMEOUT.to_string();
      return server_configs[0].clone();
    }
    
    match stream.read(&mut buf).await {
      Ok(0) => {
        // EOF reached
        append_to_file("read EOF reached").await;
        break;
      },
      Ok(n) => {
        // Successfully read n bytes from stream
        // println!("attempt to read {} bytes from stream", n);
        headers_buffer.extend_from_slice(&buf[..n]);
        // println!("after read headers buffer size: {}", headers_buffer.len());
        // println!("after read headers buffer: {:?}", headers_buffer);
        // println!("after read headers buffer to string: {:?}", String::from_utf8(headers_buffer.clone()));
        // Check if the end of the stream has been reached
        if n < buf.len() {
          append_to_file("read EOF reached relatively, because buffer not full after read").await;
          break;
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Stream is not ready yet, try again later
        continue;
      },
      Err(e) => {
        // Other error occurred
        eprintln!("ERROR: Reading headers from stream: {}", e);
        *global_error_string = ERROR_400_HEADERS_READING_STREAM.to_string();
        return server_configs[0].clone();
      },
    }
    
    
    if headers_buffer.ends_with(b"\r\n\r\n") {
      append_to_file("HEADERS BUFFER ENDS WITH \\r\\n\\r\\n").await;
      break;
    }
  }
  
  append_to_file(&format!(
    "HEADERS_BUFFER_STRING:\n{:?}",
    String::from_utf8(headers_buffer.clone())
  )).await;
  
  // chose the server_config and check the server_config.client_body_size
  // response 413 error, if body is bigger.
  // Duplicated fragment of code, because of weird task requirements.
  
  let server_config = server_config_from_headers_buffer_or_use_default(
    headers_buffer,
    server_configs.clone()
  ).await;
  
  // check of the body length, according to server_config.client_body_size.
  let client_body_size = server_config.client_body_size;
  
  // not nice
  let dirty_string = String::from_utf8_lossy(&headers_buffer);
  let is_chunked = dirty_string.to_lowercase().contains("transfer-encoding: chunked");
  
  let has_content_length_header =
  dirty_string.to_lowercase().contains("content-length: ");

  if !has_content_length_header && !is_chunked {
    append_to_file(&format!("Neither Content-Length nor Transfer-Encoding: chunked headers found in headers_buffer. Skip body reading.")).await;
    return server_config;
  }
  
  let mut content_length = 0;
  
  if has_content_length_header { // try to parse(update default 0) content_length
    let index = match dirty_string.to_lowercase().find("content-length: "){
      Some(v) => v,
      None => {
        eprintln!("ERROR: [500] Failed to find already confirmed content-length header in headers_buffer");
        *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
        return server_config;
      }
    };

    let start = index + "content-length: ".len();

    let end = match dirty_string[start..].find("\r\n"){
      Some(v) => v,
      None => {
        eprintln!("ERROR: [500] Failed to find the end( \"\\r\\n\" ) of already confirmed content-length header in headers_buffer");
        *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
        return server_config;
      }
    };

    content_length = match dirty_string[start..start + end].trim().parse(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to parse already confirmed content-length header in headers_buffer: \n{}", e);
        *global_error_string = ERROR_400_HEADERS_FAILED_TO_PARSE.to_string();
        return server_config;
      }
    };
    
  }

  append_to_file(&format!("is_chunked: {}", is_chunked)).await;
  append_to_file(&format!("has_content_length_header: {}", has_content_length_header)).await;
  append_to_file(&format!("content_length: {}", content_length)).await;
  append_to_file(&format!("====\nstream: {:?}\nbefore dive into read body", stream)).await;
  
  // ------------------------------------
  // collect request body section
  // ------------------------------------
  
  if is_chunked {
    read_chunked(
      stream,
      body_buffer,
      client_body_size,
      timeout,
      global_error_string,
    ).await;
  } else if content_length > 0  {
    read_unchunked(
      stream,
      body_buffer,
      client_body_size,
      content_length,
      timeout,
      global_error_string,
    ).await;
  }
  
  server_config
  
}

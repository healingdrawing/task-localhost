use std::io::Write;

use mio::net::TcpStream;

pub fn write_standard_error_response_into_stream(
  stream: &mut TcpStream,
  http_status_code: http::StatusCode,
) -> std::io::Result<()> {
  let mut status = http_status_code.as_u16();
  let reason = match http_status_code.canonical_reason(){
    Some(v) => v.to_string(),
    None => {
      status = http::StatusCode::INTERNAL_SERVER_ERROR.as_u16();
      "Internal Server Error: http::StatusCode.canonical_reason() failed".to_string()
    }
  };
  let status_line = format!("HTTP/1.1 {} {}\r\n", status, reason);
  
  match stream.write_all(status_line.as_bytes()){
    Ok(_) => {},
    Err(e) => {
      eprintln!("Failed to write error response status_line into the stream: {}", e);
    }
  };
  match stream.write_all(b"\r\n"){
    Ok(_) => {},
    Err(e) => {
      eprintln!("Failed to write error response empty line into the stream: {}", e);
    }
  };
  match stream.write_all(reason.as_bytes()){
    Ok(_) => {},
    Err(e) => {
      eprintln!("Failed to write error response reason into the stream: {}", e);
    }
  };
  
  Ok(())
}

use futures::AsyncWriteExt;

use async_std::net::TcpStream;

pub async fn write_critical_error_response_into_stream(
  stream: &mut TcpStream,
  http_status_code: http::StatusCode,
){
  let status = http_status_code.as_u16();
  let reason = "Internal Server Error: edge case".to_string();
  let status_line = format!("HTTP/1.1 {} {}\r\n", status, reason);
  
  match stream.write_all(status_line.as_bytes()).await{
    Ok(_) => {},
    Err(e) => {
      eprintln!("ERROR: Failed write critical error response into stream\nFailed to write error response status_line into the stream: {}", e);
    }
  };
  match stream.write_all(b"\r\n").await{
    Ok(_) => {},
    Err(e) => {
      eprintln!("ERROR: Failed write critical error response into stream\nFailed to write error response empty line into the stream: {}", e);
    }
  };
  match stream.write_all(reason.as_bytes()).await{
    Ok(_) => {},
    Err(e) => {
      eprintln!("ERROR: Failed write critical error response into stream\nFailed to write error response reason into the stream: {}", e);
    }
  };

}

use http::Response;
use async_std::net::TcpStream;
// use std::io::Write;
use futures::AsyncWriteExt;

use crate::stream::write_error::write_critical_error_response_into_stream;

pub async fn write_response_into_stream(stream: &mut TcpStream, response: Response<Vec<u8>>) -> std::io::Result<()> {
  
  // Break down the response into its parts
  let (parts, mut body) = response.into_parts();
  
  // manage errors
  let mut status: http::StatusCode ;
  match parts.status {
    http::StatusCode::INTERNAL_SERVER_ERROR // 500
    | http::StatusCode::PAYLOAD_TOO_LARGE // 413
    | http::StatusCode::METHOD_NOT_ALLOWED // 405
    | http::StatusCode::NOT_FOUND // 404
    | http::StatusCode::FORBIDDEN // 403 //todo: implement it. ERROR_403_FORBIDDEN
    | http::StatusCode::BAD_REQUEST // 400
    => {
      status = parts.status;
    },
    _ => { // force to 200
      status = http::StatusCode::OK;
      // Also force simplify any other cases to list above,
      // to satisfy the task requirements, nothing more
    }
  }
  
  let mut reason:String = match status.canonical_reason(){
    Some(v) => v.to_string(),
    None => {
      status = http::StatusCode::INTERNAL_SERVER_ERROR;
      "Internal Server Error: http::StatusCode.canonical_reason() failed".to_string()
    },
  };
  
  // Format the headers
  let mut headers = String::new();
  for (name, value) in parts.headers.iter(){
    let name = name.as_str();
    let value = match value.to_str(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to convert header value to str: {}", e);
        status = http::StatusCode::INTERNAL_SERVER_ERROR;
        reason = "Internal Server Error: incorrect header value".to_string();
        headers.push_str(&format!("{}: {}\r\n", "Content-Type", "text/plain"));
        body.extend_from_slice(b"\n\nInternal Server Error: incorrect header value");
        break;
      }
    };
    headers.push_str(&format!("{}: {}\r\n", name, value));
  }
  
  let status_line = format!("HTTP/1.1 {} {}\r\n", status, reason);
  
  // Write the status line, headers, and body to the stream
  let mut data = Vec::new();
  data.extend_from_slice(status_line.as_bytes());
  data.extend_from_slice(headers.as_bytes());
  data.extend_from_slice(b"\r\n");
  data.extend_from_slice(&body);
  match stream.write_all(data.as_slice()).await{
    Ok(_) => {},
    Err(e) => {
      eprintln!("ERROR: Failed to write response into the stream: {}", e);
      write_critical_error_response_into_stream(stream,
        http::StatusCode::INTERNAL_SERVER_ERROR,
      ).await;
      return Err(e);
    }
  };
  
  Ok(())
}

use http::Response;
use mio::net::TcpStream;
use std::io::Write;

use crate::stream::write_error::write_standard_error_response_into_stream;

pub fn write_response_into_stream(stream: &mut TcpStream, response: Response<Vec<u8>>) -> std::io::Result<()> {
  
  println!("=== write_response_into_stream: {:?}", response); //todo: remove dev print
  
  println!("response body string: {:?}", std::str::from_utf8(&response.body()).unwrap()); //todo: remove dev print
  
  // Break down the response into its parts
  let (parts, body) = response.into_parts();
  
  // Convert the body into a byte slice
  // let body_bytes = body; //todo: wtf is it? looks like crap from phind.com
  
  //todo: here probably some check for the response code, and if it is not 200, then write the error response into the stream, according to the prebuilded error pages in the server_config. Not implemented yet. Also extend function incoming parameters with server_config, to get the error pages from it. NOT LOOKS NICE, NEED RETHINK IT.
  
  
  // manage errors
  let mut status: u16 = parts.status.as_u16();
  match parts.status {
    http::StatusCode::INTERNAL_SERVER_ERROR // 500
    | http::StatusCode::PAYLOAD_TOO_LARGE // 413
    | http::StatusCode::METHOD_NOT_ALLOWED // 405
    | http::StatusCode::NOT_FOUND // 404
    | http::StatusCode::FORBIDDEN // 403
    | http::StatusCode::BAD_REQUEST // 400
    => {
      match write_standard_error_response_into_stream(stream, parts.status){
        Ok(_) => {return Ok(());},
        Err(e) => {
          eprintln!("Failed to write standard error response into the stream: {}", e);
          return Err(e);
        }
      };
    },
    _ => { // force to 200
      if parts.status.as_u16() != 200{ // should not happen, but just in case
        println!("Force status code to 200");
        status = http::StatusCode::OK.as_u16();
      }
      // Also force simplify any other cases to list above, to satisfy the task requirements, nothing more
    }
  }
  
  // let is say, the status code is 200, so try to write the response into the stream

  let mut reason:String = match parts.status.canonical_reason(){
    Some(v) => v.to_string(),
    None => {
      match write_standard_error_response_into_stream(stream,
        http::StatusCode::INTERNAL_SERVER_ERROR,
      ){
        Ok(_) => { return Ok(()) },
        Err(e) => {
          eprintln!("Failed to write error 500 response into the stream: {}", e);
          return Err(e);
        }
      }
    },
  };
  
  let mut status_line = format!("HTTP/1.1 {} {}\r\n", status, reason);
  
  // Format the headers
  // let headers = parts.headers.iter().map(|(name, value)| format!("{}: {}\r\n", name.as_str(), value.to_str().unwrap())).collect::<String>();
  // the same as above, but handle the error case, and use the for loop instead of map
  let mut headers = String::new();
  for (name, value) in parts.headers.iter(){
    let name = name.as_str();
    let value = match value.to_str(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("Failed to convert header value to str: {}", e);
        match write_standard_error_response_into_stream(stream,
          http::StatusCode::INTERNAL_SERVER_ERROR,
        ){
          Ok(_) => { return Ok(()) },
          Err(e) => {
            eprintln!("Failed to write error 500 response into the stream: {}", e);
            return Err(e);
          }
        }
      }
    };
    headers.push_str(&format!("{}: {}\r\n", name, value));
  }
  
  // Write the status line, headers, and body to the stream
  let mut data = Vec::new();
  data.extend_from_slice(status_line.as_bytes());
  data.extend_from_slice(headers.as_bytes());
  data.extend_from_slice(b"\r\n");
  data.extend_from_slice(&body);
  match stream.write_all(data.as_slice()){
    Ok(_) => {},
    Err(e) => {
      eprintln!("Failed to write response into the stream: {}", e);
      match write_standard_error_response_into_stream(stream,
        http::StatusCode::INTERNAL_SERVER_ERROR,
      ){
        Ok(_) => {},
        Err(e) => {
          eprintln!("Failed to write standard error response into the stream: {}", e);
          return Err(e);
        }
      };
      return Err(e);
    }
  };
  
  // todo: next refactor to manage ? operators above more safe. to prevent panics
  
  Ok(())
}

// todo: implement error responses for all cases, required in the task. probably add new parameter to the function above , to pass the error code. Then check it and write the error response into the stream. If the error code is 200, but in time of write to response happens some fail then write the 500 error code response into the stream.

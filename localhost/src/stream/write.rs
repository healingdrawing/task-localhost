use mio::net::TcpStream;
use std::io::Write;
use http::Response;

pub fn write_response_into_stream(stream: &mut TcpStream, response: Response<Vec<u8>>) -> std::io::Result<()> {
  
  //todo: here probably some check for the response code, and if it is not 200, then write the error response into the stream, according to the prebuilded error pages in the server_config. Not implemented yet. Also extend function incoming parameters with server_config, to get the error pages from it. NOT LOOKS NICE, NEED RETHINK IT.
  
  // the response code is 200, so write the response into the stream
  
  // Break down the response into its parts
  let (parts, body) = response.into_parts();
  
  // Convert the body into a byte slice
  let body_bytes = body; //todo: wtf is it? looks like crap from phind.com
  
  // Format the status line
  let mut status: u16 = parts.status.as_u16();
  let mut reason:String = match parts.status.canonical_reason(){
    Some(v) => v.to_string(),
    None => {
      status = 500;
      http::StatusCode::INTERNAL_SERVER_ERROR.canonical_reason().unwrap().to_string()
    },
  };

  let status_line = format!("HTTP/1.1 {} {}\r\n", status, reason);
  
  // Format the headers
  let headers = parts.headers.iter().map(|(name, value)| format!("{}: {}\r\n", name.as_str(), value.to_str().unwrap())).collect::<String>();
  
  // Write the status line, headers, and body to the stream
  stream.write_all(status_line.as_bytes())?;
  stream.write_all(headers.as_bytes())?;
  stream.write_all(&body_bytes)?;
  
  Ok(())
}

// todo: implement error responses for all cases, required in the task. probably add new parameter to the function above , to pass the error code. Then check it and write the error response into the stream. If the error code is 200, but in time of write to response happens some fail then write the 500 error code response into the stream.

/// dev gap , not tested yet
fn write_500_error_response_into_stream(stream: &mut TcpStream) -> std::io::Result<()> {
  let response = Response::builder()
  .status(500)
  .body("Internal Server Error".into())
  .unwrap();

  write_response_into_stream(stream, response)
}

use std::str;
use http::{Request, Method, Uri, Version, HeaderMap, HeaderValue, HeaderName};

/// Function to parse a raw HTTP request from a Vec<u8> buffer into an http::Request
pub fn parse_raw_request(buffer: Vec<u8>) -> Result<Request<Vec<u8>>, Box<dyn std::error::Error>> {
  if buffer.is_empty() {
    return Err("parse_request very first check: No data received(buffer is empty)".into());
  }
  
  // rust is fucking crap, for shiteaters. Rust creators must disappear.
  // Tried to convert Vec<u8> into [u8], because str::from_utf8() needs this. And ... no ways.
  // The .as_slice() method is danger(has official issues) and raise linter error.
  // They said something like ... "we plan to fix it, later..."
  // Full success .
  // ... of course the 01-edu professional hobbyists requires do not use
  // crates(another insane therminology of rust) which implement server futures
  
  let mut global_index: usize = 0; //because of rust is the future(i hope no, and rust will be r.i.p as possible fast), we will calculate indices every time for every step, ... manually.
  // Convert the buffer to a string
  let request_str = String::from_utf8(buffer).unwrap();
  
  // Split the request string into lines
  // let mut lines = request_str.lines(); //todo: never use this fucking crap. it is dead for approach more complex than hello\nworld
  
  // separate raw request to ... pieces as vector
  let rust_is_shit = request_str.split('\n');
  let mut lines: Vec<String> = Vec::new();
  for line in rust_is_shit{
    lines.push(line.to_string());
  }
  
  // Initialize a new HeaderMap to store the HTTP headers
  let mut headers = HeaderMap::new();
  // Initialize a Vec to store the request body
  let mut body = Vec::new();
  
  
  // Parse the request line, which must be the first one
  let request_line: String = match lines.get(global_index) {
    Some(value) => {value.to_string()},
    None => { return Err("fucking rust".into()) },
  };
  
  let (method, uri, version) = parse_request_line(&request_line).unwrap();
  
  // Parse the headers
  for line_index in 1..lines.len() {
    global_index += 1;
    let line: String = match lines.get(line_index){
      Some(value) => {value.to_string()},
      None => { return Err("fucking rust inside for".into()) },
    };
    
    if line.is_empty() {
      break;
    }
    
    let line2: String = match lines.get(line_index){
      Some(value) => {value.to_string()},
      None => { return Err("fucking rust inside for again".into()) },
    };
    
    let parts: Vec<String> = line2.splitn(2, ": ").map(|s| s.to_string()).collect();
    if parts.len() == 2 {
      let header_name = match HeaderName::from_bytes(parts[0].as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err("Invalid header name".into()),
      };
      println!("parsed header_name: {}", header_name);
      println!("raw header value parts[1]: {}", parts[1]);
      println!("raw header value len: {}", parts[1].len());
      let value = HeaderValue::from_str( parts[1].trim());
      match value {
        Ok(v) => headers.insert(header_name, v),
        Err(e) =>
        {
          println!("{}", e);
          return Err("Invalid header value".into())
        },
      };
      
    }
  }
  
  // Parse the body
  let mut remaining_lines:Vec<String> = Vec::new();
  for line_index in global_index..lines.len(){
    let line = match lines.get(line_index){
      Some(value) => {value},
      None => { return Err("fucking rust inside second for".into()) },
    };
    remaining_lines.push(line.to_string());
  }
  
  
  let binding = remaining_lines .join("\n");
  let remaining_bytes = binding.trim().as_bytes();
  
  body.extend_from_slice(remaining_bytes);
  
  // Construct the http::Request object
  let mut request = Request::builder()
  .method(method)
  .uri(uri)
  .version(version)
  .body(body)?;
  
  // try to fill the headers, because in builder it looks like there is no method
  // to create headers from HeaderMap, but may be force replacement can be used too
  let request_headers = request.headers_mut();
  // request_headers.clear();//todo: not safe, maybe some default must present
  for (key,value) in headers{
    let header_name = match key {
      Some(v) => v,
      None => return Err("Invalid header name".into()),
    };
    
    request_headers.append(header_name, value);
  }
  
  Ok(request)
  
}

use std::str::FromStr;
/// parse the request line into its components
fn parse_request_line(request_line: &str) -> Result<(Method, Uri, Version), Box<dyn std::error::Error>> {
  println!("raw request_line: {:?}", request_line); //todo: remove dev print
  
  let parts:Vec<&str> = request_line.trim().split_whitespace().collect();
  if parts.len() != 3 {
    return Err(format!("Invalid raw request line: {:?}", parts).into());
  }

  let (method, uri, version) = (parts[0], parts[1], parts[2]);

  let method = Method::from_str(method)
  .map_err(|e| format!("Invalid method: {} | {}", method, e))?;
  
  let uri = Uri::from_str(uri)
  .map_err(|e| format!("Invalid uri: {} | {}", uri, e))?;
  
  if version.to_ascii_uppercase() != "HTTP/1.1" {
    return Err(format!("Invalid version: {} . According to task requirements it must be HTTP/1.1 \"It is compatible with HTTP/1.1 protocol.\" ", version).into());
  }

  println!("PARSED method: {:?}, uri: {:?}, version: {:?}", method, uri, version); //todo: remove dev print
  
  Ok((method, uri, Version::HTTP_11))

}

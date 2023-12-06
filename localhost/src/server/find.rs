use http::{Request, Response, StatusCode, HeaderMap, HeaderName, HeaderValue};
use mio::net::TcpStream;
use std::io:: Write;
use std::path::PathBuf;
use std::error::Error;

use crate::server;
use crate::server::core::ServerConfig;
use crate::handlers::handle_cgi::handle_cgi;
use crate::handlers::handle_all::handle_all;
use crate::stream::errors::{ERROR_400_HEADERS_BUFFER_IS_EMPTY, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_400_HEADERS_BUFFER_TO_STRING, ERROR_500_INTERNAL_SERVER_ERROR, ERROR_400_HEADERS_INVALID_HEADER_NAME, ERROR_400_HEADERS_INVALID_HEADER_VALUE, ERROR_400_HEADERS_LINES_IS_EMPTY};
use crate::stream::parse::parse_request_line;
use crate::stream::write_::write_response_into_stream;

pub fn server_config(
  request: &Request<Vec<u8>>,
  server_configs: Vec<ServerConfig>
) -> ServerConfig{
  // choose the server config, based on the server_name and port pair of the request,
  // or use "default" , as task requires
  let mut server_config = server_configs[0].clone(); // default server config
  let request_server_host  = match request.headers().get("host"){
    Some(value) => {
      match value.to_str(){
        Ok(v) => v.to_string(),
        Err(e) => {
          eprintln!("Failed to convert request host header value \"{:?}\" to str: {}.\n=> USE \"default\" server config with first port", value, e); //todo: remove dev print. Probably
          server_config.server_name.clone() + ":" + &server_config.ports[0]
        }
      }
    },
    None => { 
      eprintln!("Fail to get request host.\n=> USE \"default\" server config with first port"); //todo: remove dev print. Probably
      server_config.server_name.clone() + ":" + &server_config.ports[0]
    },
  };
  
  println!("REQUEST_SERVER_HOST: {}", request_server_host); //todo: remove dev print
  
  // iterate server configs and the matching one will be used, two variants possible:
  // match serverconfig.server_name + ":" + &serverconfig.ports[x](for each port) == request_server_host
  // match server_config.server_address + ":" + &server_config.ports[x](for each port) == request_server_host
  for config in server_configs{
    let server_name = config.server_name.to_owned();
    let server_address = config.server_address.to_owned();
    for port in config.ports.clone(){
      let name_port_host = server_name.to_owned() + ":" + &port;
      // println!("NAME_PORT_HOST: {}", name_port_host); //todo: remove dev print
      let address_port_host = server_address.to_owned() + ":" + &port;
      if name_port_host == request_server_host
      || address_port_host == request_server_host
      {
        server_config = config.clone();
        break;
      }
    }
  }
  // println!("CHOOSEN server_config: {:?}", server_config.clone()); //todo: remove dev print

  server_config

}

pub fn server_config_from_headers_buffer_or_use_default(
  headers_buffer: &Vec<u8>,
  server_configs: Vec<ServerConfig>
) -> ServerConfig{
  
  let mut server_config = server_configs[0].clone(); // default server config

  if headers_buffer.is_empty() {
    eprintln!("server_config_from_headers: headers_buffer is empty");
    return server_config
  }

  let headers_string = match String::from_utf8( headers_buffer.clone() ){
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to convert headers_buffer to string:\n {}", e);
      return server_config
    }
  };

  // Split the request string into lines
  // let mut lines = request_str.lines(); //todo: never use this crap. it is dead for approach more complex than hello\nworld
  
  // separate raw request to ... pieces as vector
  let mut headers_lines: Vec<String> = Vec::new();
  for line in headers_string.split('\n'){ headers_lines.push(line.to_string()); }
  
  if headers_lines.is_empty() {
    eprintln!("headers_lines is empty");
    return server_config
  }

  // Initialize a new HeaderMap to store the HTTP headers
  let mut headers = HeaderMap::new();
  
  // Parse the request line, which must be the first one
  let request_line: String = match headers_lines.get(0) {
    Some(value) => {value.to_string()},
    None => {
      eprintln!("Fail to get request_line");
      return server_config
    },
  };
  
  let (method, uri, version) = match parse_request_line(request_line.clone()){
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to parse request_line: {}", e);
      return server_config
    }
  };

  // Parse the headers
  for line_index in 1..headers_lines.len() {
    // global_index += 1;
    let line: String = match headers_lines.get(line_index){
      Some(value) => {value.to_string()},
      None => {
        eprintln!("Fail to get header line");
        return server_config
      },
    };
    
    if line.is_empty() { break } //expect this can be the end of headers section
    
    let parts: Vec<String> = line.splitn(2, ": ").map(|s| s.to_string()).collect();
    if parts.len() == 2 {
      let header_name = match HeaderName::from_bytes(parts[0].as_bytes()) {
        Ok(v) => v,
        Err(e) =>{
          eprintln!("Invalid header name: {}\n {}", parts[0], e);
          return server_config
        },
      };
      // println!("parsed header_name: {}", header_name); //todo: remove dev print
      // println!("raw header value parts[1]: {}", parts[1]);
      // println!("raw header value len: {}", parts[1].len());
      let value = HeaderValue::from_str( parts[1].trim());
      match value {
        Ok(v) => headers.insert(header_name, v),
        Err(e) =>{
          eprintln!("Invalid header value: {}\n {}", parts[1], e);
          return server_config
        },
      };
      
    }
  }
  
  let body_buffer: Vec<u8> = Vec::new(); // just a gap, to fill builder
  // Construct the http::Request object
  let mut request = match Request::builder()
  .method(method)
  .uri(uri)
  .version(version)
  .body(body_buffer){
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to construct the http::Request object: {}", e);
      return server_config
    }
  };
  
  // try to fill the headers, because in builder it looks like there is no method
  // to create headers from HeaderMap, but maybe force replacement can be used too
  let request_headers = request.headers_mut();
  // request_headers.clear();//todo: not safe, maybe some default must present
  for (key,value) in headers{
    let header_name = match key {
      Some(v) => v,
      None => {
        eprintln!("Invalid header name");
        return server_config
      },
    };
    
    request_headers.append(header_name, value);
  }

  // choose the server config, based on the server_name and port pair of the request,
  // or use "default" , as task requires
  let mut server_config = server_configs[0].clone(); // default server config
  let request_server_host  = match request.headers().get("host"){
    Some(value) => {
      match value.to_str(){
        Ok(v) => v.to_string(),
        Err(e) => {
          eprintln!("Failed to convert request host header value \"{:?}\" to str: {}.\n=> USE \"default\" server config with first port", value, e); //todo: remove dev print. Probably
          server_config.server_name.clone() + ":" + &server_config.ports[0]
        }
      }
    },
    None => { 
      eprintln!("Fail to get request host.\n=> USE \"default\" server config with first port"); //todo: remove dev print. Probably
      server_config.server_name.clone() + ":" + &server_config.ports[0]
    },
  };
  
  println!("REQUEST_SERVER_HOST: {}", request_server_host); //todo: remove dev print
  
  // iterate server configs and the matching one will be used, two variants possible:
  // match serverconfig.server_name + ":" + &serverconfig.ports[x](for each port) == request_server_host
  // match server_config.server_address + ":" + &server_config.ports[x](for each port) == request_server_host
  for config in server_configs{
    let server_name = config.server_name.to_owned();
    let server_address = config.server_address.to_owned();
    for port in config.ports.clone(){
      let name_port_host = server_name.to_owned() + ":" + &port;
      // println!("NAME_PORT_HOST: {}", name_port_host); //todo: remove dev print
      let address_port_host = server_address.to_owned() + ":" + &port;
      if name_port_host == request_server_host
      || address_port_host == request_server_host
      {
        server_config = config.clone();
        break;
      }
    }
  }
  // println!("CHOOSEN server_config: {:?}", server_config.clone()); //todo: remove dev print

  server_config
}
use http::{Request, Response};
use mio::net::TcpStream;
use std::io:: Write;
use std::path::PathBuf;

use crate::server::ServerConfig;
use crate::handlers::handle_cgi::handle_cgi;
use crate::handlers::handle_all::handle_all;
use crate::stream::write_::write_response_into_stream;

/// just for test
pub fn handle_request(
  zero_path_buf: PathBuf,
  request: Request<Vec<u8>>,
  stream: &mut TcpStream,
  server_configs: Vec<ServerConfig>
) {
  
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
  
  // iterate server configs and the matching one will be used, two variants possible:
  // match serverconfig.server_name + ":" + &serverconfig.ports[x](for each port) == request_server_host
  // match server_config.server_address + ":" + &server_config.ports[x](for each port) == request_server_host
  for config in server_configs{
    let server_name = config.server_name.to_owned();
    let server_address = config.server_address.to_owned();
    for port in config.ports.clone(){
      let name_port_host = server_name.to_owned() + ":" + &port;
      let address_port_host = server_address.to_owned() + ":" + &port;
      if name_port_host == request_server_host || address_port_host == request_server_host{
        server_config = config.clone();
        break;
      }
    }
  }
  println!("CHOOSEN server_config: {:?}", server_config.clone()); //todo: remove dev print
  
  // todo!("handle_request: implement the logic. but first refactor to handle unwrap() more safe. to prevent panics");
  
  // try to manage the cgi request case strictly and separately,
  // to decrease vulnerability, because cgi is old, unsafe and not recommended to use.
  // Also, the task is low quality, because audit question ask only to check
  // the cgi with chunked and unchunked requests, so method check is not implemented,
  // because according to HTTP/1.1 standard, a not POST method can have body too
  let path = request.uri().path();
  let parts: Vec<&str> = path.split('/').collect();
  
  let response:Response<Vec<u8>> = match parts.as_slice(){
    ["", "cgi", "useless.py", file_path @ ..] => {
      handle_cgi(
        zero_path_buf,
        "useless.py".to_string(),
        file_path.join(&std::path::MAIN_SEPARATOR.to_string()),
        request,
        server_config,
      )
    },
    _ => {
      // todo : implement the response for other cases
      handle_all( zero_path_buf, request, server_config, )
      // dummy_200_response()
    }
  };
  
  match write_response_into_stream(stream, response){
    Ok(_) => println!("Response sent"),
    Err(e) => eprintln!("Failed to send response: {}", e),
  }
  
  match stream.flush(){
    Ok(_) => println!("Response flushed"),
    Err(e) => eprintln!("Failed to flush response: {}", e),
  };
  
  match stream.shutdown(std::net::Shutdown::Both) {
    Ok(_) => println!("Connection closed successfully"),
    Err(e) => eprintln!("Failed to close connection: {}", e),
  }
  
}

/// todo: remove dev gap
fn dummy_200_response() -> Response<Vec<u8>>{
  let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World! dummy_200_response\n\n";
  Response::new(response.as_bytes().to_vec())
}
use http::Request;
use mio::net::TcpStream;
use std::io:: Write;

use crate::server::ServerConfig;

/// just for test
pub fn handle_request(request: Request<Vec<u8>>, stream: &mut TcpStream, server_configs: Vec<ServerConfig>) {

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

  todo!("handle_request: implement the logic. but first refactor to handle unwrap() more safe. to prevent panics");

  // For simplicity, just send a "Hello, World!" response
  let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";

  match stream.write_all(response.as_bytes()){
    Ok(_) => println!("Response sent"),
    Err(e) => eprintln!("Failed to send response: {}", e),
  };
  
  match stream.flush(){
    Ok(_) => println!("Response flushed"),
    Err(e) => eprintln!("Failed to flush response: {}", e),
  };

}
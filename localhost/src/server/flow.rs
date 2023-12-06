use http::{StatusCode, Response, Request};
use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;
use std::error::Error;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::handlers::response_::check_custom_errors;
use crate::handlers::response_4xx::{self, custom_response_4xx};
use crate::handlers::handle_::handle_request;

use crate::server::core::{get_usize_unique_ports, Server};
use crate::server::core::ServerConfig;

use crate::stream::errors::ERROR_200_OK;
use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::stream::write_::write_response_into_stream;


/// in exact run the server implementation, after all settings configured properly
pub fn run(zero_path_buf:PathBuf ,server_configs: Vec<ServerConfig>) {
  
  let ports = match get_usize_unique_ports(&server_configs){
    Ok(v) => v,
    Err(e) => panic!("ERROR: Failed to get_unique_ports: {}", e),
  };
  
  // to listen on all interfaces, then redirect to pseudo servers by server_name like task requires
  let server_address = "0.0.0.0";
  
  let mut servers = Vec::new();
  
  for port in ports {
    let addr: SocketAddr = match 
    format!("{}:{}", server_address, port).parse(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to parse socket address: {} | {}", format!("{}:{}", server_address, port), e);
        continue;
      }
    };
    
    let listener = match TcpListener::bind(addr){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to bind to socket address: {} | {}", addr, e);
        continue;
      },
    };
    servers.push(Server { listener, token: Token(port) });
    
  }
  
  let mut poll = match Poll::new(){
    Ok(v) => v,
    Err(e) => panic!("ERROR: Failed to create Poll: {}", e),
  };
  
  let mut events = Events::with_capacity(1024);
  
  for server in servers.iter_mut() {
    match poll.registry().register(&mut server.listener, server.token, Interest::READABLE){
      Ok(v) => v,
      Err(e) => panic!("ERROR: Failed to register server.listener: {}", e),
    };
    
  }
  
  println!("CONFIGURED:\n{:?}\n", servers);
  println!(
    "====================\n= START the server =\n===================="
  );
  
  loop {
    // poll.poll(&mut events, Some(Duration::from_millis(100))).unwrap(); // changes nothing
    match poll.poll(&mut events, None){
      Ok(v) => v,
      Err(e) => {
        eprint!("ERROR: Failed to poll: {}", e);
        continue;
      },
    };
    
    for event in events.iter() {
      
      println!("event: {:?}", event);
      
      let token = event.token();
      
      // Find the server associated with the token
      let server = match servers.iter_mut().find(|s| s.token.0 == token.0){
        Some(v) => v,
        None => {
          eprintln!("ERROR: Failed to find server by token: {}", token.0);
          continue;
        }
      };
      
      println!("server: {:?}", server);
      
      // Accept the incoming connection
      let (mut stream, _) = match server.listener.accept(){
        Ok(v) => v,
        Err(e) => { // according to docs it can be io::ErrorKind::WouldBlock, so just continue
          eprintln!("ERROR: Failed to accept incoming connection: {}", e);
          continue;
        }
      };
      
      println!("stream: {:?}", stream); //todo: remove dev print
      
      // create buffers here and fill them with read_with_timeout
      let timeout = Duration::from_millis(5000);
      let mut headers_buffer: Vec<u8> = Vec::new();
      let mut body_buffer: Vec<u8> = Vec::new();
      
      // use first server config as default
      let mut choosen_server_config = server_configs[0].clone();
      let mut global_error_string = ERROR_200_OK.to_string();
      
      println!("=== choosen_server_config: {:?}", choosen_server_config); //todo: remove dev print
      
      let mut response:Response<Vec<u8>> = Response::new(Vec::new());
      
      // Read the HTTP request from the client
      read_with_timeout(
        timeout,
        &mut stream,
        &mut headers_buffer,
        &mut body_buffer,
        &mut choosen_server_config,
        server_configs.clone(),
        &mut global_error_string,
      );
      
      println!("=== updated choosen_server_config:\n{:?}", choosen_server_config); //todo: remove dev print
      
      println!("Buffer sizes after read: headers_buffer: {}, body_buffer: {}", headers_buffer.len(), body_buffer.len()); //todo: remove dev print
      
      if headers_buffer.is_empty() {
        println!("NO DATA RECEIVED, empty headres_buffer");
      }else if body_buffer.is_empty() {
        println!("NO DATA RECEIVED, empty body_buffer");
      }else{
        println!("buffers are not empty"); //todo: remove dev print
        println!("Raw buffers:\nheaders_buffer:\n=\n{}\n=\nbody_buffer:\n=\n{}\n=", String::from_utf8_lossy(&headers_buffer), String::from_utf8_lossy(&body_buffer));
      }
      
      let mut request = Request::new(Vec::new());

      if global_error_string == ERROR_200_OK.to_string() {
        
        parse_raw_request(
          headers_buffer,
          body_buffer,
          &mut request,
          &mut global_error_string,
        );
        println!("request: {:?}", request); //todo: remove dev print
        
        // let server_config = server_config(&request, server_configs.clone());
        
        response = handle_request(
          &request,
          zero_path_buf.clone(),
          choosen_server_config.clone(),
          &mut global_error_string,
        );
        
      }

      check_custom_errors(
        global_error_string,
        &request,
        zero_path_buf.clone(),
        choosen_server_config.clone(),
        &mut response,
      );
      
      match write_response_into_stream(&mut stream, response){
        Ok(_) => println!("Response sent"),
        Err(e) => {
          eprintln!("Failed to send response: {}", e)
          //todo: remove the stream from poll registry some way
        },
      }
      
      match stream.flush(){
        Ok(_) => println!("Response flushed"),
        Err(e) => {
          eprintln!("Failed to flush response: {}", e)
          //todo: remove the stream from poll registry some way
        },
      };
      
      match stream.shutdown(std::net::Shutdown::Both) {
        Ok(_) => println!("Connection closed successfully"),
        Err(e) => {
          eprintln!("Failed to close connection: {}", e)
          //todo: remove the stream from poll registry some way
        },
      }
      
    }
  }
  
}
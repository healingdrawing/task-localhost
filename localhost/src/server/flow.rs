use http::StatusCode;
use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;
use serde::Deserialize;
use core::num;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::handlers::response_4xx::{self, custom_response_4xx};
use crate::handlers::handle_::handle_request;

use crate::server::core::{get_usize_unique_ports, Server};
use crate::server::core::ServerConfig;

use crate::server::find::server_config;
use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::stream::write_::write_response_into_stream;


/// in exact run the server implementation, after all settings configured properly
pub fn run(zero_path_buf:PathBuf ,server_configs: Vec<ServerConfig>) {
  
  let ports = match get_usize_unique_ports(&server_configs){
    Ok(v) => v,
    Err(e) => panic!("Failed to get_unique_ports: {}", e),
  };
  
  // to listen on all interfaces, then redirect to pseudo servers by server_name like task requires
  let server_address = "0.0.0.0";
  
  let mut servers = Vec::new();
  
  for port in ports {
    let addr: SocketAddr = match 
    format!("{}:{}", server_address, port).parse(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("Failed to parse socket address: {} | {}", format!("{}:{}", server_address, port), e);
        continue;
      }
    };
    
    let listener = match TcpListener::bind(addr){
      Ok(v) => v,
      Err(e) => {
        eprintln!("Failed to bind to socket address: {} | {}", addr, e);
        continue;
      },
    };
    servers.push(Server { listener, token: Token(port) });

  }
  
  let mut poll = match Poll::new(){
    Ok(v) => v,
    Err(e) => panic!("Failed to create Poll: {}", e),
  };

  let mut events = Events::with_capacity(1024);
  
  for server in servers.iter_mut() {
    match poll.registry().register(&mut server.listener, server.token, Interest::READABLE){
      Ok(v) => v,
      Err(e) => panic!("Failed to register server.listener: {}", e),
    };

  }
  
  println!("CONFIGURED:\n{:?}", servers);
  println!("====================\n= START the server =\n====================");

  loop {
    // poll.poll(&mut events, Some(Duration::from_millis(100))).unwrap(); // changes nothing
    match poll.poll(&mut events, None){
      Ok(v) => v,
      Err(e) => {
        eprint!("Failed to poll: {}", e);
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
          eprintln!("Failed to find server by token: {}", token.0);
          continue;
        }
      };
      
      println!("server: {:?}", server);
      
      // Accept the incoming connection
      let (mut stream, _) = match server.listener.accept(){
        Ok(v) => v,
        Err(e) => {
          eprintln!("Failed to accept incoming connection: {}", e);
          continue;
        }
      };
      
      println!("stream: {:?}", stream); //todo: remove dev print
      
      //todo: refactor to create buffers here and fill them with read_with_timeout instead of creating buffers in read_with_timeout

      let timeout = Duration::from_millis(5000);
      let mut headers_buffer: Vec<u8> = Vec::new();
      let mut body_buffer: Vec<u8> = Vec::new();

      // Read the HTTP request from the client
      match
      read_with_timeout( timeout, &mut stream, &mut headers_buffer, &mut body_buffer ){
        Ok(v) => v,
        Err(e) => {
          eprintln!("Failed to read from stream: {:?} {}", stream, e);
        }
      };
      
      println!("Buffer sizes after read: headers_buffer: {}, body_buffer: {}", headers_buffer.len(), body_buffer.len()); //todo: remove dev print
      
      if headers_buffer.is_empty() {
        println!("NO DATA RECEIVED, empty headres_buffer");
      }else if body_buffer.is_empty() {
        println!("NO DATA RECEIVED, empty body_buffer");
      }else{
        println!("buffers are not empty"); //todo: remove dev print
        println!("Raw buffers:\nheaders_buffer:\n=\n{}\n=\nbody_buffer:\n=\n{}\n=", String::from_utf8_lossy(&headers_buffer), String::from_utf8_lossy(&body_buffer));
      }
      
      let request = match parse_raw_request(headers_buffer, body_buffer) {
        Ok(request) => request,
        Err(e) => {
          eprintln!("Failed to parse request: {}", e);
          //todo: send 400 response some way
          continue;
        }
      };
      println!("request: {:?}", request);

      //todo: implement chose server_config based on request host header
      let server_config = server_config(&request, server_configs.clone());

      // Handle the request and send a response
      // handle_request(zero_path_buf.clone(), request, &mut stream, server_configs.clone());

      let response = match handle_request(
        &request,
        zero_path_buf.clone(),
        server_config.clone()
      ){
        Ok(v) => v,
        Err(e) => {
          eprintln!("Failed to handle request: {}", e);
          //todo: send 400 response some way
          custom_response_4xx(
            &request,
            StatusCode::BAD_REQUEST,
            zero_path_buf.clone(),
            server_config)
          
        }
      };
      
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
use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;
use serde::Deserialize;
use core::num;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::handlers::handle_::handle_request;


#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub server_name: String,
  pub ports: Vec<String>,
  pub server_address: String,
  pub client_body_size: usize,
  pub static_files_prefix: String,
  pub default_file: String,
  pub error_pages_prefix: String,
  pub routes: HashMap<String, Route>,
}

impl ServerConfig {
  /// drop out all ports not in 0..65535 range, also drop out all repeating of ports
  pub fn check(&mut self){
    self.check_ports();
  }
  
  /// drop out all ports not in 0..65535 range, also drop out all repeating of ports
  fn check_ports(&mut self){
    let old_ports = self.ports.clone();
    let mut ports: HashSet<String> = HashSet::new();
    for port in self.ports.iter(){
      let port: u16 = match port.parse(){
        Ok(v) => v,
        Err(e) =>{
          eprintln!("Config \"{}\" Failed to parse port: {} into u16", self.server_name, e);
          continue;
        }
      };
      ports.insert(port.to_string());
    }
    self.ports = ports.into_iter().collect();
    if self.ports.len() != old_ports.len(){
      eprintln!("=== Config \"{}\" ports changed\nfrom {:?}\n  to {:?}", self.server_name, old_ports, self.ports);
    }
    
  }
  
}

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
  methods: Vec<String>,
}

#[derive(Debug)]
struct Server {
  listener: TcpListener,
  token: Token,
}

/// create token usize from ip:port string
fn ip_port_to_token(server: &Server) -> Result<usize, String>{
  let addr = match server.listener.local_addr(){
    Ok(v) => v.to_string(),
    Err(e) => return Err(format!("Failed to get local_addr from server.listener: {}", e)),
  };
  let mut token_str = String::new();
  // accumulate token_str from addr chars usize values
  for c in addr.chars(){
    let c_str = (c as usize).to_string();
    token_str.push_str(&c_str);
  }
  
  let token: usize = match token_str.parse(){
    Ok(v) => v,
    Err(e) => return Err(format!("Failed to parse token_str: {} into usize: {}", token_str, e)),
  };
  
  Ok(token)
}

/// get list of unique ports from server_configs, to use listen 0.0.0.0:port
/// 
/// and manage pseudo servers, because the task requires redirection
/// 
/// if no declared server name in request, so we need to use "default" server
pub fn get_usize_unique_ports(server_configs: &Vec<ServerConfig>) -> Result<Vec<usize>, Box<dyn Error>>{
  let mut ports: HashSet<usize> = HashSet::new();
  for server_config in server_configs.iter(){
    for port in server_config.ports.iter(){
      let port: u16 = match port.parse(){
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to parse port: {} into u16: {}", port, e).into()),
      };
      ports.insert(port as usize);
    }
  }
  let ports: Vec<usize> = ports.into_iter().collect();
  
  if ports.len() < 1 {
    return Err("Not enough correct ports declared in \"settings\" file".into());
  }
  
  Ok(ports)
}

/// in exact run the server implementation, after all settings configured properly
pub fn run(zero_path:String ,server_configs: Vec<ServerConfig>) {
  
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
      
      // Read the HTTP request from the client
      let ( headers_buffer, body_buffer) = match
      read_with_timeout(&mut stream, Duration::from_millis(5000)){
        Ok(v) => v,
        Err(e) => {
          eprintln!("Failed to read from stream: {:?} {}", stream, e);
          continue;
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
      
      // TODO: Parse the HTTP request and handle it appropriately...
      match parse_raw_request(headers_buffer, body_buffer) {
        Ok(request) => {
          println!("request: {:?}", request);
          // Handle the request and send a response
          handle_request(zero_path.clone(), request, &mut stream, server_configs.clone());
        },
        Err(e) => eprintln!("Failed to parse request: {}", e),
      }
      
      
      
    }
  }
  
}
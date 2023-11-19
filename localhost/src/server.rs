use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::handlers::handle_::handle_request;


#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  server_name: String,
  ports: Vec<String>,
  server_address: String,
  default_file: String,
  error_pages: HashMap<String, String>,
  client_body_size: usize,
  routes: HashMap<String, Route>,
}

impl ServerConfig {
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

#[derive(Debug, Deserialize)]
struct Route {
  methods: Vec<String>,
}

#[derive(Debug)]
struct Server {
  listener: TcpListener,
  // Add other fields as needed...
  name: String, // to use "default" if request quality is as good as 01-edu tasks
}

/// in exact run the server implementation, after all settings configured properly
pub fn run(server_configs: Vec<ServerConfig>) {
  
  let mut servers = Vec::new();
  for config in server_configs {
    for port in config.ports {
      let addr: SocketAddr = 
      format!("{}:{}", config.server_address, port).parse().unwrap();
      let listener = TcpListener::bind(addr).unwrap();
      servers.push(Server { listener, name: config.server_name.clone() });
    }
  }
  
  let mut poll = Poll::new().unwrap();
  let mut events = Events::with_capacity(128);
  
  for server in servers.iter_mut() {
    let token = Token(server.listener.local_addr().unwrap().port().into()); // Use the port number as the token
    poll.registry().register(&mut server.listener, token, Interest::READABLE).unwrap();
  }
  
  loop {
    poll.poll(&mut events, None).unwrap();
    // poll.poll(&mut events, Some(Duration::from_millis(1000))).unwrap(); // changes nothing
    
    for event in events.iter() {
      
      println!("event: {:?}", event);
      
      match event.token() {
        token => {
          // Find the server associated with the token
          let server = servers.iter_mut().find(|s| s.listener.local_addr().unwrap().port() as usize == token.0).unwrap();
          
          println!("server: {:?}", server);
          
          // Accept the incoming connection
          let (mut stream, _) = server.listener.accept().unwrap();
          
          println!("stream: {:?}", stream);
          
          // Read the HTTP request from the client
          let (mut headers_buffer,mut body_buffer) = read_with_timeout(&mut stream, Duration::from_millis(5000)).unwrap(); //todo: manage it properly, server should never crash
          
          println!("Buffer sizes after read: headers_buffer: {}, body_buffer: {}", headers_buffer.len(), body_buffer.len());
          
          if headers_buffer.is_empty() {
            println!("NO DATA RECEIVED, empty headres_buffer");
          }else if body_buffer.is_empty() {
            println!("NO DATA RECEIVED, empty body_buffer");
          }else{
            println!("buffers are not empty");
            println!("Raw buffres:\nheaders_buffer:\n=\n{}\n=\nbody_buffer:\n=\n{}\n=", String::from_utf8_lossy(&headers_buffer), String::from_utf8_lossy(&body_buffer));
          }
          
          // TODO: Parse the HTTP request and handle it appropriately...
          match parse_raw_request(headers_buffer, body_buffer) {
            Ok(request) => {
              // Handle the request and send a response
              handle_request(request, &mut stream);
            },
            Err(e) => eprintln!("Failed to parse request: {}", e),
          }
          
          
        },
        _ => unreachable!(),
      }
    }
  }
  
}
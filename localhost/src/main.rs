mod debug;
use debug:: try_recreate_file_according_to_value_of_debug_boolean;
use debug::DEBUG;

mod stream{
  pub mod read;
  pub mod parse;
}
use stream::read::read_with_timeout;
use stream::parse::parse_raw_request;

use std::env;
use std::time::Duration;
use config::{ConfigError, File, FileFormat};
use serde::Deserialize;
use std::collections::HashMap;

use mio::{Events, Interest, Poll, Token};
use std::io:: Write;
use std::process::{Command, Stdio};
use mio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;

fn main() {
  println!("Hello, world!");
  match try_recreate_file_according_to_value_of_debug_boolean(){
    Ok(_) => println!("debug file recreated"),
    Err(e) => println!("debug file recreation failed: {}", e),
  }
  
  let mut config_path = match env::current_exe(){
    Ok(v) => v,
    Err(e) => panic!("Failed to get current exe path: {}", e),
  };
  config_path.pop(); // Remove the executable name from the path
  config_path.push("settings"); // Add the configuration file name to the path
  
  let mut settings = config::Config::builder();
  let config_path_str = match config_path.to_str(){
    Some(v) => v,
    None => panic!("Failed to convert config_path to str"),
  };
  
  settings = settings.add_source(
    File::new(config_path_str , FileFormat::Toml)
  );
  
  let settings = settings.build();
  
  match settings {
    Ok(config) => {
      let server_configs: Result<Vec<ServerConfig>, _> = config.get("servers");
      match server_configs {
        Ok(server_configs) =>{ // configuration read successfully
          //todo: need to implement custom check(and perhaps dropout incorrect settings), as required audit. It is about wrong configurations in "settings" file. As always the 01-edu description is as clear as brain flow of the mindset handicap. So it is some extra brain fuck, which is better to solve in advance.
          println!("{:#?}", server_configs); //todo: remove this dev print
          gogogo(server_configs);
        },
        Err(e) => eprintln!("Failed to convert settings into Vec<ServerConfig>: {}", e),
      }
    }
    Err(e) => eprintln!("Failed to build settings: {}", e),
  }
}



#[derive(Debug, Deserialize)]
struct ServerConfig {
  server_name: String,
  server_address: String,
  ports: Vec<String>,
  error_pages: HashMap<String, String>,
  client_body_size: usize,
  routes: HashMap<String, Route>,
}

#[derive(Debug, Deserialize)]
struct Route {
  methods: Vec<String>,
  cgi: String,
}

#[derive(Debug)]
struct Server {
  listener: TcpListener,
  // Add other fields as needed...
  name: String, // to use "default" if request quality is as good as 01-edu tasks
}

/// in exact run the server implementation, after all settings configured properly
fn gogogo(server_configs: Vec<ServerConfig>) {
  
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

use http::Request;

/// just for test
fn handle_request(request: Request<Vec<u8>>, stream: &mut TcpStream) {
  // For simplicity, just send a "Hello, World!" response
  let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
  stream.write_all(response.as_bytes()).unwrap();
  stream.flush().unwrap();
}
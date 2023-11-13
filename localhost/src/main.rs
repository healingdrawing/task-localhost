use std::env;
use config::{ConfigError, File, FileFormat};
use serde::Deserialize;
use std::collections::HashMap;

use mio::{Events, Interest, Poll, Token};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use mio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;


fn main() {
  println!("Hello, world!");
  
  let mut config_path = env::current_exe().unwrap();
  config_path.pop(); // Remove the executable name from the path
  config_path.push("settings"); // Add the configuration file name to the path
  
  let mut settings = config::Config::builder();
  
  settings = settings.add_source(File::new(config_path.to_str().unwrap()
  , FileFormat::Toml));
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
    
    for event in events.iter() {
      match event.token() {
        token => {
          // Find the server associated with the token
          let server = servers.iter_mut().find(|s| s.listener.local_addr().unwrap().port() as usize == token.0).unwrap();
          
          // Accept the incoming connection
          let (mut stream, _) = server.listener.accept().unwrap();
          
          // Read the HTTP request from the client
          let mut buffer = [0; 1024];
          stream.read(&mut buffer).unwrap(); //todo: need to handle this error, some limits about size
          
          // TODO: Parse the HTTP request and handle it appropriately...
        },
        _ => unreachable!(),
      }
    }
  }
  
}
use std::env;
use std::time::{Instant, Duration};
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
          let mut buffer =
          read_with_timeout(&mut stream,
            Duration::from_millis(100))
            .unwrap(); //todo: manage it properly, server should never crash
            
            // TODO: Parse the HTTP request and handle it appropriately...
          },
          _ => unreachable!(),
        }
      }
    }
    
  }
  
  fn read_with_timeout(stream: &mut TcpStream, timeout: Duration) -> io::Result<Vec<u8>> {
    
    // Start the timer
    let start_time = Instant::now();
    
    // Read from the stream until timeout or EOF
    let mut buffer = Vec::new();
    loop {
      // Check if the timeout has expired
      if start_time.elapsed() >= timeout {
        return Err(io::Error::new(io::ErrorKind::TimedOut, "read timed out"));
      }
      
      // Read from the stream
      let mut buf = Vec::with_capacity(1024);
      match stream.read(&mut buf) {
        Ok(0) => {
          // EOF reached
          break;
        },
        Ok(n) => {
          // Successfully read n bytes from stream
          buffer.extend_from_slice(&buf[..n]);
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          // Stream is not ready yet, try again later
          continue;
        },
        Err(e) => {
          // Other error occurred
          return Err(e);
        },
      }
    }
    
    Ok(buffer)
  }
  
  
  
  
  
  
  use std::str;
  use http::{Request, Method, Uri, Version, HeaderMap, HeaderValue, HeaderName};
  
  // Function to parse a raw HTTP request from a Vec<u8> buffer into an http::Request
  fn parse_request(buffer: Vec<u8>) -> Result<Request<Vec<u8>>, Box<dyn std::error::Error>> {
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
    
    let (method, uri, version) = parse_request_line(&request_line)?;
    
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
        let header_name = match HeaderName::from_lowercase(parts[0].as_bytes()) {
          Ok(v) => v,
          Err(_) => return Err("Invalid header name".into()),
        };
        let value = parts[1].parse::<HeaderValue>();
        match value {
          Ok(v) => headers.insert(header_name, v),
          Err(_) => return Err("Invalid header value".into()),
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
    for (key,value) in headers{
      let header_name = match key {
        Some(v) => v,
        None => return Err("Invalid header name".into()),
      };
      
      request_headers.append(header_name, value);
    }
    
    Ok(request)
    
  }
  
  // Function to parse the request line into its components
  fn parse_request_line(request_line: &str) -> Result<(Method, Uri, Version), Box<dyn std::error::Error>> {
    // Implement the parsing of the request line here
    // Example: Split the request line into method, uri, and version
    // Return the parsed components
    unimplemented!()
  }
  
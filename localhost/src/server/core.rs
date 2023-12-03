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
use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::handlers::handle_::handle_request;
use crate::stream::write_::write_response_into_stream;


#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub server_name: String,
  pub ports: Vec<String>,
  pub server_address: String,
  pub client_body_size: usize,
  pub static_files_prefix: String,
  pub default_file: String,
  pub error_pages_prefix: String,
  pub routes: HashMap<String, Vec<String>>,
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

// #[derive(Debug, Deserialize, Clone)]
// pub struct Route {
//   pub methods: Vec<String>,
// }

#[derive(Debug)]
pub struct Server {
  pub listener: TcpListener,
  pub token: Token,
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

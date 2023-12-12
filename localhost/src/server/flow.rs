use async_std::net::TcpListener;
// use async_std::prelude::*;
use async_std::task;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use http::{Response, Request};

use crate::handlers::response_::check_custom_errors;
use crate::handlers::handle_::handle_request;
use crate::server::core::{get_usize_unique_ports, Server};
use crate::server::core::ServerConfig;
use crate::stream::errors::{ERROR_200_OK, ERROR_400_HEADERS_INVALID_COOKIE};
use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::stream::write_::write_response_into_stream;
use crate::debug::append_to_file;

pub async fn run(zero_path_buf:PathBuf ,server_configs: Vec<ServerConfig>) {
  let ports = get_usize_unique_ports(&server_configs).unwrap();
  let server_address = "0.0.0.0";
  
  for port in ports.clone() {
    let addr: SocketAddr = format!("{}:{}", server_address, port).parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    let listener_arc = Arc::new(listener);
    let mut server = Server {
      listener: Arc::clone(&listener_arc),
      cookies: HashMap::new(),
      cookies_check_time: SystemTime::now() + Duration::from_secs(60),
    };
    
    let server_configs_clone = server_configs.clone();
    let zero_path_buf_clone = zero_path_buf.clone();
    
    let one_task = task::spawn( async move {
      append_to_file("BANG").await; // FIRES ONCE, FOR EACH PORT
      append_to_file(&format!("{:?}",listener_arc)).await; // FIRES ONCE, FOR 
      loop {
        task::sleep(Duration::from_secs(1)).await; // Add a delay of 1 second
        append_to_file("step1").await; // NEVER FIRES
        append_to_file(&format!("{:?}",listener_arc)).await; // NEVER FIRES
        
      }
    });
    one_task.await; // wait for the task to finish
    
  }
  append_to_file("Server is listening on http://{}:{}").await; // FIRES ONCE, FOR EACH PORT
  println!("Server is listening on http://{}:{}", server_address, ports[0])
}
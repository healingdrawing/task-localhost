use futures::AsyncWriteExt;
use async_std::net::TcpListener;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
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
  let ports = get_usize_unique_ports(&server_configs).await.unwrap();
  let server_address = "0.0.0.0";
  // let mut servers = Vec::new();
  
  let zero_path_buf_clone = &zero_path_buf.clone();
  let server_configs_clone = &server_configs.clone();
  for port in ports.clone() {
    let addr: SocketAddr = format!("{}:{}", server_address, port).parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    // servers.push(server);
    append_to_file(&format!("addr{}\n", addr)).await;
    
    let all = listener.incoming()
    .for_each_concurrent(/* limit */ None, |stream| async move {
      let mut stream = stream.unwrap();
      let timeout = Duration::from_millis(1000);
      let mut headers_buffer: Vec<u8> = Vec::new();
      let mut body_buffer: Vec<u8> = Vec::new();
      let mut global_error_string = ERROR_200_OK.to_string();
      let mut response:Response<Vec<u8>> = Response::new(Vec::new());
      let choosen_server_config = 
      read_with_timeout( timeout, &mut stream, &mut headers_buffer, &mut body_buffer, server_configs_clone, &mut global_error_string).await;
      let mut request = Request::new(Vec::new());
      if global_error_string == ERROR_200_OK.to_string() {
        parse_raw_request(headers_buffer, body_buffer, &mut request, &mut global_error_string).await;
      }
      let mut server = Server { cookies: HashMap::new(), cookies_check_time: SystemTime::now() + Duration::from_secs(60), };
      server.check_expired_cookies().await;

      let (cookie_value, cookie_is_ok) = server.extract_cookies_from_request_or_provide_new(&request).await;
      if !cookie_is_ok { global_error_string = ERROR_400_HEADERS_INVALID_COOKIE.to_string(); }
      
      if global_error_string == ERROR_200_OK.to_string() {
        response = handle_request(&request, cookie_value.clone(), zero_path_buf_clone, choosen_server_config.clone(), &mut global_error_string).await;
      }
      check_custom_errors(global_error_string, &request, cookie_value.clone(), zero_path_buf_clone, choosen_server_config.clone(), &mut response).await;
        write_response_into_stream(&mut stream, response).await.unwrap();
        stream.flush().await.unwrap();
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    });
    all.await;
  }
  append_to_file("Server is listening on http://{}:{}").await; // FIRES ONCE, FOR EACH PORT
  println!("Server is listening on http://{}:{}", server_address, ports[0])
}
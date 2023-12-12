use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};

use http::{Response, Request};
use crate::handlers::response_::check_custom_errors;
use crate::handlers::handle_::handle_request;
use crate::server::core::{get_usize_unique_ports, Server};
use crate::server::core::ServerConfig;
use crate::stream::errors::{ERROR_200_OK, ERROR_400_HEADERS_INVALID_COOKIE};
use crate::stream::read::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::stream::write_::write_response_into_stream;

pub async fn run(zero_path_buf:PathBuf ,server_configs: Vec<ServerConfig>) {
 let ports = get_usize_unique_ports(&server_configs).unwrap();
 let server_address = "0.0.0.0";
 let servers = Arc::new(Mutex::new(Vec::new()));

 for port in ports {
 let addr: SocketAddr = format!("{}:{}", server_address, port).parse().unwrap();
 let listener = TcpListener::bind(addr).await.unwrap();
 let mut servers = servers.lock().unwrap();
 servers.push(Server { listener: Arc::new(listener), cookies: HashMap::new(), cookies_check_time: SystemTime::now() + Duration::from_secs(60), });
 }

 let servers = Arc::clone(&servers);
 for server in servers.lock().unwrap().iter_mut() {
 let listener = server.listener.clone();
 task::spawn(async move {
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let timeout = Duration::from_millis(1000);
        let mut headers_buffer: Vec<u8> = Vec::new();
        let mut body_buffer: Vec<u8> = Vec::new();
        let mut choosen_server_config = server_configs[0].clone();
        let mut global_error_string = ERROR_200_OK.to_string();
        let mut response:Response<Vec<u8>> = Response::new(Vec::new());
        read_with_timeout(timeout, &mut stream, &mut headers_buffer, &mut body_buffer, &mut choosen_server_config, server_configs.clone(), &mut global_error_string);
        let mut request = Request::new(Vec::new());
        if global_error_string == ERROR_200_OK.to_string() {
          parse_raw_request(headers_buffer, body_buffer, &mut request, &mut global_error_string);
        }
        server.check_expired_cookies();
        let (cookie_value, cookie_is_ok) = server.extract_cookies_from_request_or_provide_new(&request);
        if !cookie_is_ok { global_error_string = ERROR_400_HEADERS_INVALID_COOKIE.to_string(); }
        if global_error_string == ERROR_200_OK.to_string() {
          response = handle_request(&request, cookie_value.clone(), zero_path_buf.clone(), choosen_server_config.clone(), &mut global_error_string);
        }
        check_custom_errors(global_error_string, &request, cookie_value.clone(), zero_path_buf.clone(), choosen_server_config.clone(), &mut response);
        write_response_into_stream(&mut stream, response).unwrap();
        // stream.flush().unwrap();
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }
 });
 }
}

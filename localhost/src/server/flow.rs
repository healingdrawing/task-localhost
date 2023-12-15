use async_std::sync::Mutex;
use async_std::task;
use futures::AsyncWriteExt;
use async_std::net::TcpListener;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use http::{Response, Request};
use std::error::Error;

use crate::handlers::response_::check_custom_errors;
use crate::handlers::handle_::handle_request;
use crate::server::cookie::Cookie;
use crate::server::core::{get_usize_unique_ports, Server};
use crate::server::core::ServerConfig;
use crate::stream::errors::{ERROR_200_OK, ERROR_400_HEADERS_INVALID_COOKIE};
use crate::stream::read_::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::stream::write_::write_response_into_stream;
use crate::debug::append_to_file;

pub async fn run(
  zero_path_buf:PathBuf,
  server_configs: Vec<ServerConfig>
) -> Result<(), Box<dyn Error>> {

  let ports = match get_usize_unique_ports(&server_configs).await{
    Ok(ports) => ports,
    Err(e) => {
      eprintln!("ERROR: Failed to get ports: {}", e);
      return Err("Failed to get ports".into());
    },
  };
  
  let server_address = "0.0.0.0"; // to listen all interfaces
  
  for port in ports.clone() {
    let addr: SocketAddr = match format!(
      "{}:{}",
      server_address,
      port,
    ).parse(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to parse 0.0.0.0:port into SocketAddr: {}", e);
        return Err("Failed to parse 0.0.0.0:port into SocketAddr".into());
      },
    };
    
    let listener = match TcpListener::bind(addr).await{
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to bind addr: {}", e);
        return Err("Failed to bind addr".into());
      },
    };
    
    append_to_file(&format!("addr {}", addr)).await;
    
    let zero_path_buf = zero_path_buf.clone();
    let server_configs = server_configs.clone();
    
    // Create an infinite stream of incoming connections for each port
    task::spawn(async move {

      // also can be one for all tasks(move outside), but this looks like more safe/isolated
      let cookies_storage: Arc<Mutex<HashMap<String, Cookie>>> =
        Arc::new(Mutex::new(HashMap::new()));
      
      listener.incoming().for_each_concurrent(None, |stream| async {
        
        let mut stream = match stream{
          Ok(v) => v,
          Err(e) => {
            eprintln!("ERROR: Failed to get stream: {}", e);
            return;
          },
        };
        
        append_to_file(
          "==================\n= incoming fires =\n=================="
        ).await;
        // append_to_file(&format!("{:?}",stream)).await;
        
        let mut server = Server {
          cookies: cookies_storage.clone(),
          cookies_check_time: SystemTime::now() + Duration::from_secs(60), };
        
        let mut headers_buffer: Vec<u8> = Vec::new();
        let mut body_buffer: Vec<u8> = Vec::new();
        let mut global_error_string = ERROR_200_OK.to_string();
        
        append_to_file(&format!( "\nbefore read_with_timeout\nheaders_buffer: {:?}", headers_buffer )).await;

        let mut response:Response<Vec<u8>> = Response::new(Vec::new());
        
        // hardcoded, but it's ok for this case. And less chance for user to break.
        // Not bad to manage it as flag of executable.
        let timeout = Duration::from_millis(5000);

        let choosen_server_config = read_with_timeout(
          timeout, &mut stream, &mut headers_buffer, &mut body_buffer,
          &server_configs, &mut global_error_string
        ).await;
        
        append_to_file(&format!(
          "\nafter read_with_timeout\nheaders_buffer_string: {:?}\nbody_buffer_string: {:?}" ,
          String::from_utf8(headers_buffer.clone()),
          String::from_utf8(body_buffer.clone())
        )).await;
        
        let mut request = Request::new(Vec::new());
        if global_error_string == ERROR_200_OK.to_string() {
          parse_raw_request(headers_buffer, body_buffer, &mut request, &mut global_error_string).await;
        }
        
        append_to_file(&format!(
          "\nafter parse_raw_request\nrequest.headers: {:?}\n" ,
          request.headers()
        )).await;

        server.check_expired_cookies().await;
        
        let (cookie_value, cookie_is_ok) = server.extract_cookies_from_request_or_provide_new(&request).await;
        
        if !cookie_is_ok { global_error_string = ERROR_400_HEADERS_INVALID_COOKIE.to_string(); }
        
        // let cookie_value = "key=gap;key2=gap2".to_string();// todo: remove this line

        if global_error_string == ERROR_200_OK.to_string() {
          response = handle_request(&request, cookie_value.clone(), &zero_path_buf, choosen_server_config.clone(), &mut global_error_string).await;
        }

        append_to_file(&format!(
          "\nafter handle_request\nresponse.headers: {:?}\n" ,
          response.headers()          
        )).await;
        
        check_custom_errors(global_error_string, &request, cookie_value.clone(), &zero_path_buf, choosen_server_config.clone(), &mut response).await;
        
        match write_response_into_stream(&mut stream, response).await{
          Ok(_) => {},
          Err(e) =>{
            eprintln!("ERROR: Failed to write response into stream: {}", e);
            return // to force drop the task stream, just for case. It will exit anyways
          },
        };
        
        match stream.flush().await{
          Ok(_) => {},
          Err(e) => {
            eprintln!("ERROR: Failed to flush stream: {}", e);
            return // to force drop the task stream, just for case. It will exit anyways
          },
        };
        
        match stream.shutdown(std::net::Shutdown::Both){
          Ok(_) => {},
          Err(e) => {
            eprintln!("ERROR: Failed to shutdown stream: {}", e);
            return // to force drop the task stream, just for case. It will exit anyways
          },
        };
        
      }).await;
    });
    
  }
  println!("Server is listening configured above http://ip:port pairs");
  async_std::task::sleep(Duration::from_secs(3600)).await;
  Ok(())
}
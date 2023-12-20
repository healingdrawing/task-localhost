use std::process::Command;
use async_std::path::PathBuf;

use http::{Request, Response, StatusCode};

use crate::server::core::ServerConfig;
use crate::handlers::response_500::custom_response_500;
use crate::handlers::response_4xx::custom_response_4xx;

/// run python script , and check the path is file,folder or not exist/wrong path
/// 
/// unsafe potentially , because you can pass any path to the script using
/// 
/// cgi/useless.py//some/path/here. but in exact this case allow only to check
pub async fn handle_cgi(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  script_file_name: String,
  check_file_path: String,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  
  // check if method is GET or POST or DELETE, or return 405
  if request.method() != "GET" && request.method() != "POST" && request.method() != "DELETE"{
    eprintln!("ERROR: Method {} is not allowed for cgi", request.method());
    return custom_response_4xx(
      request,
      cookie_value.clone(),
      zero_path_buf,
      server_config,
      StatusCode::METHOD_NOT_ALLOWED,
    ).await
  }

  let script_path = "cgi/".to_owned() + &script_file_name;
  
  // check if script still exist, else return 500, because before server start, we check mandatory files
  if !zero_path_buf.join(&script_path).exists().await{
    eprintln!("ERROR: script_path {:?} is not exist.\nThe file structure was damaged after the server started.", zero_path_buf.join(&script_path));
    return custom_response_500(
      request,
      cookie_value.clone(),
      zero_path_buf,
      server_config,
    ).await
  }

  // Set the system PATH_INFO or send request path_info into python3 script as argument
  let output = Command::new("python3")
  .arg(script_path)
  .arg(check_file_path)
  .output();
  
  let result = match &output{
    Ok(v) => match std::str::from_utf8(&v.stdout){
      Ok(v) => {
        if v.trim() == ""{
          "Empty output from cgi python3 script"
        } else {
          v
        }
      },
      Err(e) => {
        let error_message = "Failed to convert cgi output to str. ".to_owned() + &e.to_string();
        Box::leak(error_message.into_boxed_str())
      }
    },
    Err(e) => {
      let error_message = "Failed to get cgi output. ".to_owned() + &e.to_string();
      Box::leak(error_message.into_boxed_str())
    } // new puzzle. instead of return error as string(instead of coding), fuck your brain for hours , with borrowing/binding/lifetime etc, because it is just can not do this naturally. Rust is crap
  };
  
  // write to the stream
  let body = format!("Hello from Rust and Python3: {}\n\n", result)
  .as_bytes().to_vec();
  
  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/plain")
  .header("Set-Cookie", cookie_value.clone())
  .body(body)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to build cgi response body | {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config,
      ).await
    }
    
  };
  
  response
  
}

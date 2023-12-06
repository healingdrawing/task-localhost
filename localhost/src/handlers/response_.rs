use std::error::Error;
use std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::{server::core::ServerConfig, handlers::response_500::custom_response_500};
use crate::stream::errors::{CUSTOM_ERRORS_400, CUSTOM_ERRORS_413, CUSTOM_ERRORS_500};

use super::response_4xx::custom_response_4xx;


/// create response with static file, according to server config
pub fn response_default_static_file(
  zero_path_buf: PathBuf,
  request: &Request<Vec<u8>>,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  let default_file_path = zero_path_buf
  .join("static")
  .join(server_config.static_files_prefix.clone())
  .join(server_config.default_file.clone());
  println!("default_file_path {:?}", default_file_path); //todo: remove dev print

  // read the default file. if error, then return error response with 500 status code,
  // because before server start, all files checked, so it is server error
  let default_file_content = match std::fs::read(default_file_path){
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to read default file: {}", e); //todo: remove dev print
      return custom_response_500(
        request, 
        zero_path_buf, 
        server_config
      )
    }
  };

  let mut response = match Response::builder()
  .status(StatusCode::OK)
  .body(default_file_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to create response with default file: {}", e);
      return custom_response_500(
        request, 
        zero_path_buf, 
        server_config
      )
    }
  };
  
  response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());

  response
}

//todo: implement check error and return response respectivelly, based on arrays of custom errors in errors.rs

/// check error and return response respectivelly, based on arrays of custom errors in errors.rs
pub fn check_custom_errors(
  custom_error_string: String,
  request: &Request<Vec<u8>>,
  zero_path_buf: PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  
  // check error 400 array
  for error in CUSTOM_ERRORS_400.iter(){
    if custom_error_string == *error{
      return custom_response_4xx(
        request,
        zero_path_buf,
        server_config,
        StatusCode::BAD_REQUEST
      )
    }
  }

  // check error 413
  for error in CUSTOM_ERRORS_413.iter(){
    if custom_error_string == *error{
      return custom_response_4xx(
        request,
        zero_path_buf,
        server_config,
        StatusCode::PAYLOAD_TOO_LARGE
      )
    }
  }

  // check error 500. Actually it can be just return custom_response_500, without check. No difference at the moment
  for error in CUSTOM_ERRORS_500.iter(){
    if custom_error_string == *error{
      return custom_response_500(
        request,
        zero_path_buf,
        server_config,
      )
    }
  }

  // if error not found, then return custom 500 response
  custom_response_500(
    request,
    zero_path_buf,
    server_config,
  )

}

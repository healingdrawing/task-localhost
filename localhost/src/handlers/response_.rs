use std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::files::check::ERROR_PAGES;

use crate::handlers::response_500::custom_response_500;
use crate::handlers::response_4xx::custom_response_4xx;

use crate::server::core::ServerConfig;

use crate::stream::errors::{CUSTOM_ERRORS_400, CUSTOM_ERRORS_413};
use crate::stream::errors::{CUSTOM_ERRORS_500, ERROR_200_OK};


/// create response with static file, according to server config
pub fn response_default_static_file(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  let default_file_path = zero_path_buf
  .join("static")
  .join(server_config.static_files_prefix.clone())
  .join(server_config.default_file.clone());
  
  // read the default file. if error, then return error response with 500 status code,
  // because before server start, all files checked, so it is server error
  let default_file_content = match std::fs::read(default_file_path){
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to read default file: {}", e); //todo: remove dev print
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      )
    }
  };
  
  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(default_file_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to create response with default file: {}", e);
      return custom_response_500(
        request, 
        cookie_value.clone(),
        zero_path_buf,
        server_config,
      )
    }
  };
  
  response
}

/// check error and return response respectivelly, based on arrays of custom errors in errors.rs
pub async fn check_custom_errors(
  custom_error_string: String,
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
  response: &mut Response<Vec<u8>>,
) {
  
  if custom_error_string != ERROR_200_OK.to_string(){
    
    // check error 400 array
    for error in CUSTOM_ERRORS_400.iter(){
      if custom_error_string == *error{
        *response = custom_response_4xx(
          request,
          cookie_value,
          zero_path_buf,
          server_config.clone(),
          StatusCode::BAD_REQUEST
        );
        return
      }
    }
    
    // check error 413
    for error in CUSTOM_ERRORS_413.iter(){
      if custom_error_string == *error{
        *response = custom_response_4xx(
          request,
          cookie_value,
          zero_path_buf,
          server_config.clone(),
          StatusCode::PAYLOAD_TOO_LARGE
        );
        return
      }
    }
    
    // check error 500. Actually it can be just return custom_response_500, without check. No difference at the moment
    for error in CUSTOM_ERRORS_500.iter(){
      if custom_error_string == *error{
        *response = custom_response_500(
          request,
          cookie_value,
          zero_path_buf,
          server_config.clone(),
        );
        return
      }
    }
    
    // if error not found, then return custom 500 response
    *response = custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config.clone(),
    )
  }
  
}

/// check the path ends to find error pages, and return response respectivelly, or return 200 OK
/// 
/// it is needed for manual testing/requesting of error pages
pub fn force_status(
  zero_path_buf: PathBuf,
  absolute_path_buf: PathBuf,
  server_config: ServerConfig,
)-> StatusCode {
  
  let error_pages_prefix = server_config.error_pages_prefix.clone();
  
  // check if path ends with error pages prefix
  for error_page in ERROR_PAGES.iter(){
    
    let error_path = zero_path_buf
    .join("static")
    .join(&error_pages_prefix)
    .join(error_page);
    
    if absolute_path_buf == error_path{
      
      return match error_page{
        &"400.html" => StatusCode::BAD_REQUEST,
        &"403.html" => StatusCode::FORBIDDEN,
        &"404.html" => StatusCode::NOT_FOUND,
        &"405.html" => StatusCode::METHOD_NOT_ALLOWED,
        &"413.html" => StatusCode::PAYLOAD_TOO_LARGE,
        &"500.html" => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::OK, // should never happen
      }

    }
    
  }
  
  StatusCode::OK
}
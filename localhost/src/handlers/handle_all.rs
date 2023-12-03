use std::path::{Path, PathBuf};

use http::{Response, Request, StatusCode};

use crate::handlers::response_500::custom_response_500;
use crate::server::core::{ServerConfig, self};
use crate::handlers::response_::response_default_static_file;
use crate::handlers::response_4xx::custom_response_4xx;


/// handle all requests, except cgi.
/// 
/// Also, in case of uri is directory, the task requires to return default file,
/// according to server config. So in this case, there is no need to check the method,
/// allowed for route.
pub fn handle_all(
  zero_path_buf: PathBuf,
  request: &Request<Vec<u8>>,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  // todo: refactor path check to os separator instead of hardcoding of / ... probably
  
  // analyze path. if path is directory, then return default file, according to server config
  let mut path = request.uri().path();
  // cut first slash
  if path.starts_with("/"){ path = &path[1..]; }
  println!("path {}", path); // todo: remove dev prints
  // path to site folder in static folder
  let relative_static_site_path = format!("static/{}/{}", server_config.static_files_prefix, path);
  println!("relative_static_site_path {}", relative_static_site_path);
  let absolute_path = zero_path_buf.join(relative_static_site_path);
  println!("absolute_path {:?}", absolute_path);
  
  // check if path is directory, then return default file as task requires
  if path.ends_with("/") || absolute_path.is_dir() {
    return response_default_static_file( zero_path_buf, request, server_config, );
  } else if !absolute_path.is_file() {
    return custom_response_500(
      request, 
      zero_path_buf, 
      server_config)
  } // check if file exists or return 500, because before server start, all files checked, so it is server error. The "uploads" folder managed separately with 404
  
  
  let parts: Vec<&str> = path.split('/').collect();
  println!("=== parts {:?}", parts); // todo: remove dev prints
  
  // check if path is inside routes, then get methods allowed for this path
  let allowed_methods = match server_config.routes.get(path){
    Some(v) => {v},
    None => {
       return custom_response_4xx(
        request,
        http::StatusCode::NOT_FOUND,
        zero_path_buf,
        server_config,
       )
    }
  };

  // check if method is allowed for this path or return 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    return custom_response_4xx(
      request,
      http::StatusCode::METHOD_NOT_ALLOWED,
      zero_path_buf,
      server_config,
    )
  }

  // read the file. if error, then return error 500 response
  let file_content = match std::fs::read(absolute_path.clone()){
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to read file: {}", e); //todo: remove dev print
      return custom_response_500(
        request,
        zero_path_buf,
        server_config
      )
    }
  };

  let mut response = match Response::builder()
  .status(StatusCode::OK)
  .body(file_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to create response with file: {}", e);
      return custom_response_500(
        request,
        zero_path_buf,
        server_config)
    }
  };

  // get file mime type using mime_guess, or use the text/plain
  let mime_type = match mime_guess::from_path(absolute_path.clone()).first(){
    Some(v) => v.to_string(),
    None => "text/plain".to_string(),
  };
  println!("mime_type {}", mime_type); //todo: remove dev print

  response.headers_mut().insert(
    "Content-Type",
    match mime_type.parse(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("Failed to parse mime type: {}", e);
        "text/plain".parse().unwrap()
      }
    }
  );
  
  response
  
}
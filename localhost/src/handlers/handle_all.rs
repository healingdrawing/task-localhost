use std::path::PathBuf;

use http::{Response, Request, StatusCode, HeaderValue};

use crate::debug::append_to_file;
use crate::files::check::is_implemented_error_page;
use crate::handlers::response_500::custom_response_500;
use crate::handlers::response_::{response_default_static_file, force_status};
use crate::handlers::response_4xx::custom_response_4xx;
use crate::server::core::ServerConfig;


/// handle all requests, except cgi, and except uploads.
/// 
/// Also, in case of uri is directory, the task requires to return default file,
/// according to server config. So in this case, there is no need to check the method,
/// allowed for route.
pub async fn handle_all(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  // todo: refactor path check to os separator instead of hardcoding of / ... probably
  
  // replace /uploads/ to /, to prevent wrong path. The uploads files served separately on the upper level
  let binding_path_string = request.uri().path().replacen("uploads/", "", 1);
  let mut path_str = binding_path_string.as_str();
  
  // cut first slash
  if path_str.starts_with("/"){ path_str = &path_str[1..]; }
  
  // check if path is error page
  let is_error_page = is_implemented_error_page(path_str);
  // path to site folder in static folder
  let relative_static_path_string =
  if is_error_page {
    
    let file_name = match path_str.split('/').last(){
      Some(v) => v,
      None => {
        eprintln!("ERROR: path_str.split('/').last()\nFailed with path {}", path_str);
        eprintln!(" Must never fire, because path checked/confirmed before.\nSo return [500]");
        return custom_response_500(
          request,
          cookie_value,
          zero_path_buf,
          server_config,
        ).await
      }
    };
    format!("static/{}/{}", server_config.error_pages_prefix, file_name)
  }
  else { format!("static/{}/{}", server_config.static_files_prefix, path_str)};
  
  let absolute_path_buf = zero_path_buf.join(relative_static_path_string);
  
  // check if path is directory, then return default file as task requires
  if path_str.ends_with("/") || absolute_path_buf.is_dir() {
    
    // implement 403 error check if method is not GET, to satisfy task requirements
    if request.method().to_string() != "GET" {
      return custom_response_4xx(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
        StatusCode::FORBIDDEN,
      ).await
    }
    
    return response_default_static_file(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    ).await
  } else if !absolute_path_buf.is_file() {
    
    eprintln!("ERROR:\n------------\nIS NOT A FILE\n-------------");
    
    return custom_response_4xx(
      request, 
      cookie_value,
      zero_path_buf, 
      server_config,
      StatusCode::NOT_FOUND,
    ).await
  } // check if file exists or return 404
  
  // check if path is inside routes, then get methods allowed for this path
  let mut rust_handicap_binding:Vec<String> = Vec::new();
  let allowed_methods: &Vec<String> = match server_config.routes.get(path_str){
    Some(v) => {v},
    None => {
      if is_error_page {
        rust_handicap_binding.push("GET".to_string());
        &rust_handicap_binding
        
      } else {
        eprintln!("ERROR: Path {} is not inside routes", path_str);
        return custom_response_4xx(
          request,
          cookie_value,
          zero_path_buf,
          server_config,
          http::StatusCode::NOT_FOUND,
        ).await
      }
    }
  };
  
  // check if method is allowed for this path or return 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    eprintln!("ERROR: Method {} is not allowed for path {}", request_method_string, path_str);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    ).await
  }
  
  // read the file. if error, then return error 500 response
  let file_content = match std::fs::read(absolute_path_buf.clone()){
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to read file: {}", e);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config
      ).await
    }
  };
  
  let mut response = match Response::builder()
  .status(
    force_status(
      zero_path_buf.clone(),
      absolute_path_buf.clone(),
      server_config.clone(),
    )
  )
  .header("Set-Cookie", cookie_value.clone())
  .body(file_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to create response with file: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config
      ).await
    }
  };
  
  // get file mime type using mime_guess, or use the text/plain
  let mime_type = match mime_guess::from_path(absolute_path_buf.clone()).first(){
    Some(v) => v.to_string(),
    None => "text/plain".to_string(),
  };
  append_to_file(&format!("\n-------\n\nmime_type {}\n\n----------\n", mime_type)).await;
  
  response.headers_mut().insert(
    "Content-Type",
    match mime_type.parse(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to parse mime type: {}", e);
        HeaderValue::from_static("text/plain")
      }
    }
  );
  
  response
  
}

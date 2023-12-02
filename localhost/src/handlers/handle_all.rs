use std::path::{Path, PathBuf};

use http::{Response, Request};

use crate::{server::ServerConfig, handlers::response_::response_default_static_file};


/// handle all requests, except cgi.
/// 
/// Also, in case of uri is directory, the task requires to return default file,
/// according to server config. So in this case, there is no need to check the method,
/// allowed for route.
pub fn handle_all(
  zero_path_buf: PathBuf,
  request: Request<Vec<u8>>,
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
  
  let parts: Vec<&str> = path.split('/').collect();
  
  // check if path is directory, then return default file
  if path.ends_with("/") || absolute_path.is_dir() {
    // return default file in response
    let response = response_default_static_file(
      zero_path_buf,
      request,
      server_config,
    );
    return response;
  }
  
  let result = "DEV GAP";
  let body = format!("Hello from Rust and Python3: {}\n\n", result);
  let mut response = Response::new(body.as_bytes().to_vec());
  response.headers_mut().insert("Content-Type", "text/plain".parse().unwrap());
  
  response
  
}
use std::path::PathBuf;

use http::{Response, Request};

use crate::server::ServerConfig;


/// create response with static file, according to server config
pub fn response_default_static_file(
  absolute_path: PathBuf,
  request: Request<Vec<u8>>,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{

  let mut response = Response::new("DUMMY GAP OF DEFAULT FILE \n\n".as_bytes().to_vec());
  response.headers_mut().insert("Content-Type", "text/plain".parse().unwrap());

  response
}
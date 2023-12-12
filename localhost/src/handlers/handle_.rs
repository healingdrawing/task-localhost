use http::{Request, Response};
use std::path::PathBuf;

use crate::server::core::ServerConfig;
use crate::handlers::handle_cgi::handle_cgi;
use crate::handlers::handle_all::handle_all;
use crate::handlers::handle_uploads::handle_uploads;
use crate::handlers::uploads_get::handle_uploads_get_uploaded_file;


/// handle all requests.
/// The cgi requests are handled like separated match case.
/// The uploads requests are handled separated match case.
pub fn handle_request(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
  global_error_string: &mut String, //at the moment not mutated here
) -> Response<Vec<u8>>{
  
  // try to manage the cgi request case strictly and separately,
  // to decrease vulnerability, because cgi is old, unsafe and not recommended to use.
  // Also, the task is low quality, because audit question ask only to check
  // the cgi with chunked and unchunked requests, so method check is not implemented,
  // because according to HTTP/1.1 standard, a not POST method can have body too
  let path = request.uri().path();
  let parts: Vec<&str> = path.split('/').collect();
  
  let response = match parts.as_slice(){
    ["", "cgi", "useless.py", file_path @ ..] => {
      handle_cgi(
        request,
        cookie_value,
        zero_path_buf,
        "useless.py".to_string(),
        file_path.join(&std::path::MAIN_SEPARATOR.to_string()),
        server_config,
      )

    },
    ["", "uploads"] => {
      handle_uploads(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      )
      
    },
    ["", "uploads", file_path ] => {
      handle_uploads_get_uploaded_file(
        request,
        cookie_value,
        zero_path_buf,
        file_path.to_string(),
        server_config,
      )
    },
    _ => {
      // todo : implement the response for other cases
      handle_all(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      )
    }
  };
  
  response
}

// todo: remove dev gap
// fn dummy_200_response() -> Response<Vec<u8>>{
//   let body = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World! dummy_200_response\n\n";
//   Response::builder()
//   .status(StatusCode::OK)
//   .body(body.as_bytes().to_vec())
//   .unwrap()
// }

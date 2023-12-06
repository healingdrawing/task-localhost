use std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::server::core::ServerConfig;

/// last point, if custom error 500 response failed
fn hardcoded_response_500(
  request: &Request<Vec<u8>>, //probably for cookies stuff
) -> Response<Vec<u8>>{

  println!("\n\nhardcoded_response_500"); //todo: remove dev print

  let body = "Hardcoded status 500. Internal Server Error. Custom error 500 response failed. \n\n".as_bytes().to_vec();
  
  let mut response = match Response::builder()
  .status(StatusCode::INTERNAL_SERVER_ERROR)
  .header("Content-Type", "text/plain")
  .body(body)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to create hardcoded 500 response: {}", e);
      return Response::new("Fatal Internal Server Error.\nFailed to create hardcoded error 500 response.\nStatus 500 does not set properly".as_bytes().to_vec());
    }
  };
  
  response

}

/// return custom 500 error response.
/// if error happens, then return hardcoded response with 500 status code
pub fn custom_response_500(
  request: &Request<Vec<u8>>,
  zero_path_buf: PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{

  println!("\n\ncustom_response_500"); //todo: remove dev print

  let error_page_path = zero_path_buf.join("static").join(server_config.error_pages_prefix.clone()).join("500.html");
  println!("\n\nerror_page_path {:?}", error_page_path); //todo: remove dev print
  
  // read the error page. if error, then return hardcoded response with 500 status code
  let error_page_content = match std::fs::read(error_page_path){
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to read error page: {}", e); //todo: remove dev print
      return hardcoded_response_500(request)
    }
  };

  let print_error_page_content = std::str::from_utf8(&error_page_content).unwrap(); //todo: remove dev print
  println!("\n\nerror_page_content {:?}", print_error_page_content); //todo: remove dev print
  
  let response = match Response::builder()
  .status(StatusCode::INTERNAL_SERVER_ERROR)
  .header("Content-Type", "text/html")
  .body(error_page_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to create custom 500 response: {}", e);
      return hardcoded_response_500(request)
    }
  };

  response
}

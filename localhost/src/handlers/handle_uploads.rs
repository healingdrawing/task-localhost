use std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::handlers::response_500::custom_response_500;
use crate::server::core::ServerConfig;
use crate::handlers::response_4xx::custom_response_4xx;


/// handle uploads requests.
/// 
/// The task requires to implement the uploads requests handling.
/// 
/// Method GET, POST, DELETE are allowed for
/// 
/// GET - to get the generated in code dynamic html page, includes
/// the list of files in uploads folder, click on file send DELETE request
/// and form to upload new file
/// 
/// POST - to upload new file, using form from previous GET request
/// 
/// DELETE - to delete the file, using form from previous GET request.
/// Left mouse click on file, will send the DELETE request to server.
pub fn handle_uploads(
  zero_path_buf: PathBuf,
  request: &Request<Vec<u8>>,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  // todo: refactor path check to os separator instead of hardcoding of / ... probably
  
  let mut path = request.uri().path();
  // cut first slash
  if path.starts_with("/"){ path = &path[1..]; }
  println!("path {}", path); // todo: remove dev prints
  
  let absolute_path = zero_path_buf.join("uploads");
  println!("absolute_path {:?}", absolute_path);// todo: remove dev prints
  
  // check if path is directory, then return default file as task requires
  if !absolute_path.is_dir() {
    
    eprintln!("------------\nIS NOT A FOLDER\n-------------"); // todo: remove dev print
    eprintln!("ERROR: absolute_path {:?} is not a folder.\nThe file structure was damaged after the server started.", absolute_path);
    
    return custom_response_500(
      request,
      zero_path_buf,
      server_config
    )
  } // check if file exists or return 404
  
  
  let parts: Vec<&str> = path.split('/').collect();
  println!("=== parts {:?}", parts); // todo: remove dev prints
  
  // methods allowed for this path, according to task, GET, POST, DELETE
  let allowed_methods:Vec<String> = vec![
  "GET".to_string(),
  "POST".to_string(),
  "DELETE".to_string(),
  ];
  
  // check if method is allowed for this path or return 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    return custom_response_4xx(
      request,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    )
  }
  
  // new implementation
  let mut body_content:Vec<u8> = Vec::new();
  
  match request_method_string.as_str(){
    "GET" => {
      body_content.extend_from_slice(b"GET uploads\n");
      // todo: implement generating of the dynamic html page to return as response body. The page includes the list of files in uploads folder(click on file send DELETE request), and the from to upload new file, with POST request, after pressing the "upload file" button.
    },
    "POST" => {
      body_content.extend_from_slice(b"POST uploads\n");
      // todo: implement the file upload, using the form from GET request, and return the dynamic html page with the list of files in uploads folder(click on file send DELETE request), and the from to upload new file, with POST request, after pressing the "upload file" button.
    },
    "DELETE" => {
      body_content.extend_from_slice(b"DELETE uploads\n");
      // todo: implement the file delete, using the form from GET request, and return the dynamic html page with the list of files in uploads folder(click on file send DELETE request), and the from to upload new file, with POST request, after pressing the "upload file" button.
    },
    _ => {
      eprintln!("ERROR: method {} is not implemented for path {}.\nShould never fire, because checked above!!!", request_method_string, path);
      return custom_response_500(
        request,
        zero_path_buf,
        server_config
      )
    },
  }
  
  // the old code section starts here, // todo: refactor it to the new code section
  // read the file. if error, then return error 500 response
  // let body_content = match std::fs::read(absolute_path.clone()){
  //   Ok(v) => v,
  //   Err(e) => {
  //     eprintln!("Failed to read file: {}", e); //todo: remove dev print
  //     return custom_response_500(
  //       request,
  //       zero_path_buf,
  //       server_config
  //     )
  //   }
  // };
  
  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/html")
  .body(body_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Failed to create response with file: {}", e);
      return custom_response_500(
        request,
        zero_path_buf,
        server_config
      )
    }
  };
  
  response
  
}

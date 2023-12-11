use std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::handlers::response_500::custom_response_500;
use crate::handlers::uploads_delete::delete_the_file_from_uploads_folder;
use crate::handlers::uploads_get::generate_uploads_html;
use crate::handlers::uploads_set::upload_the_file_into_uploads_folder;
use crate::server::core::ServerConfig;
use crate::handlers::response_4xx::custom_response_4xx;


/// handle uploads requests.
/// 
/// The task requires to implement the uploads requests handling.
/// 
/// Method GET, POST, DELETE are allowed for
/// 
/// GET - to get the generated in code dynamic html page, includes
/// the list of files in uploads folder, with [Delete] button to send DELETE request,
/// and [Upload] button, with form to upload new file
/// 
/// POST - to upload new file, using form from previous GET request
/// 
/// DELETE - to delete the file, using form from previous GET request.
/// Press [Delete] button, to send the DELETE request to server.
pub fn handle_uploads(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: PathBuf,
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
    
    // eprintln!("------------\nIS NOT A FOLDER\n-------------"); // todo: remove dev print
    eprintln!("ERROR: absolute_path {:?} is not a folder.\nThe file structure was damaged after the server started.", absolute_path);
    
    return custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config
    )
  } // check if file exists or return 404
  
  
  let parts: Vec<&str> = path.split('/').collect();
  // println!("=== parts {:?}", parts); // todo: remove dev prints
  
  // methods allowed for this path, according to task, GET, POST, DELETE
  let allowed_methods:Vec<String> = vec![
  "GET".to_string(),
  "POST".to_string(),
  "DELETE".to_string(),
  ];
  
  // check if method is allowed for this path or return 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    eprintln!("ERROR: method {} is not allowed for uploads", request_method_string);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    )
  } else if !server_config.uploads_methods.contains(&request_method_string){
    eprintln!("ERROR: method {} is not allowed for uploads in server_config", request_method_string);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    );
  }
  
  
  let mut body_content:Vec<u8> = Vec::new();
  
  match request_method_string.as_str(){
    "GET" => { /* do nothing unique. The html page is generated below */ },
    "POST" => { upload_the_file_into_uploads_folder(request, &absolute_path); },
    "DELETE" => { delete_the_file_from_uploads_folder(request, &absolute_path); },
    _ => {
      eprintln!("ERROR: method {} is not implemented for path {}.\nShould never fire, because checked above!!!", request_method_string, path);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config
      )
    },
  }
  
  body_content.extend_from_slice(
    generate_uploads_html( &absolute_path, ).as_bytes(),
  );

  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(body_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to create response with body_content: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config
      )
    }
  };
  
  response
  
}

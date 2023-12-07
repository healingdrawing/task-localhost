use std::{path::PathBuf, fs};

use http::{Request, Response, StatusCode};

use crate::{server::core::ServerConfig, handlers::{response_::response_default_static_file, response_4xx::custom_response_4xx, response_500::custom_response_500}};

/// html is generated in code. Not templates etc.
/// 
/// To decrease dependencies and avoid any extra activities.
pub fn generate_uploads_html(absolute_path: &PathBuf) -> String {
  let mut html = String::new();
  html.push_str("<h1>Uploads</h1>");
  html.push_str("<ul>");
  for entry in fs::read_dir(absolute_path).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_file() {
      let file_name = path.file_name().unwrap().to_str().unwrap();
      if file_name == ".gitignore" {// gap to do not collect crap in repo, and in same time keep the folders structure
        continue;
      }
      html.push_str(&format!("<li><a href=\"/uploads/{}\">{}</a>", file_name, file_name));
      html.push_str(&format!("<button onclick=\"deleteFile('{}')\">Delete</button>", file_name));
      html.push_str("</li>");
    }
  }
  html.push_str("</ul>");
  html.push_str("<form method=\"POST\" action=\"/uploads\">");
  html.push_str("<input type=\"file\" name=\"file\">");
  html.push_str("<input type=\"submit\" value=\"Upload\">");
  html.push_str("</form>");
  html.push_str("<script>");
  html.push_str("function deleteFile(fileName) {");
  html.push_str(" fetch('/uploads', {");
  html.push_str("  method: 'DELETE',");
  html.push_str("  headers: {");
  html.push_str("    'Content-Type': 'application/x-www-form-urlencoded',");
  html.push_str("  },");
  html.push_str("  body: 'file=' + encodeURIComponent(fileName),");
  html.push_str(" }).then(() => {");
  html.push_str("  location.reload();");
  html.push_str(" });");
  html.push_str("}");
  html.push_str("</script>");
  html
}


/// when some file requested from uploads folder,
/// 
/// this function manages processing of this request.
/// 
/// Managed separately, because the uploads folder is dynamic. Not safe to use.
pub fn handle_uploads_get_uploaded_file(
  zero_path_buf: PathBuf,
  file_path: String,
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
  
  let absolute_path = zero_path_buf.join("uploads").join(file_path);
  println!("absolute_path {:?}", absolute_path);
  
  // check if path is directory, then return default file as task requires
  if path.ends_with("/") || absolute_path.is_dir() {
    println!("path is dir. Handle uploads file"); // todo: remove dev print
    return response_default_static_file( zero_path_buf, request, server_config, );
  } else if !absolute_path.is_file() {
    
    eprintln!("ERROR:\n------------\nIS NOT A FILE\n-------------");
    
    return custom_response_4xx(
      request, 
      zero_path_buf, 
      server_config,
      StatusCode::NOT_FOUND,
    )
  } // check if file exists or return 404
  
  
  let parts: Vec<&str> = path.split('/').collect();
  println!("=== parts {:?}", parts); // todo: remove dev prints
  
  // only GET method allowed for this path. filtering happens above
  let allowed_methods = vec!["GET".to_string()];
  
  // check if method is allowed for this path or return 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    eprintln!("ERROR: method {} is not allowed for path {}", request_method_string, path);
    return custom_response_4xx(
      request,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    )
  }
  
  // read the file. if error, then return error 500 response
  let file_content = match std::fs::read(absolute_path.clone()){
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to read file: {}", e);
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
      eprintln!("ERROR: Failed to create response with file: {}", e);
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
    println!("\n-------\n\nmime_type {}\n\n----------\n", mime_type); //todo: remove dev print
    
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
  
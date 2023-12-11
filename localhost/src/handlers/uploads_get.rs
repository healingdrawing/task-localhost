use std::{path::PathBuf, fs};

use http::{Request, Response, StatusCode};

use crate::server::core::ServerConfig;
use crate::handlers::response_::response_default_static_file;
use crate::handlers::response_4xx::custom_response_4xx;
use crate::handlers::response_500::custom_response_500;
use crate::files::check::bad_file_name;


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
      
      if bad_file_name(file_name) {
        eprintln!("ERROR: bad file name \"{}\" inside \"uploads\" folder.\nPotential crappers activity :|,\nor file_name sanitised not properly\n", file_name);
        continue;
      }
      if file_name == ".gitignore" { continue; }
      
      html.push_str("\n<li>");
      html.push_str(&format!("\n<button onclick=\"deleteFile('{}')\">Delete</button>", file_name));
      html.push_str(&format!("\n<a href=\"/uploads/{}\">{}</a>", file_name, file_name));
      html.push_str("\n</li>");
    }
  }
  html.push_str("\n</ul>");
  
  let form = r#"
  <form method="POST" action="/uploads" id="uploadForm" enctype="multipart/form-data">
  <input type="file" name="file" id="fileInput">
  <input type="submit" value="Upload">
  </form>
  "#;
  
  html.push_str(&form);
  
  let script = r#"
  <script>
  function deleteFile(fileName) {
    fetch('/uploads', {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: 'file=' + encodeURIComponent(fileName),
      redirect: 'manual'
    }).then(response => {
      console.log(response.status);
      if (!response.ok) {
        if (response.status === 400) {
          console.log('400 crap piles from 01 delivered');
          window.location.href = '400.html';
        }
        else if (response.status === 403) {
          console.log('403 crap piles from 01 delivered');
          window.location.href = '403.html';
        }
        else if (response.status === 404) {
          console.log('404 crap piles from 01 delivered');
          window.location.href = '404.html';
        }
        else if (response.status === 405) {
          console.log('405 Method Not Allowed');
          window.location.href = '405.html';
        }
        else if (response.status === 413) {
          console.log('413 crap piles from 01 delivered');
          window.location.href = '413.html';
        }
        else {
          console.log('500 crap piles from 01 delivered');
          window.location.href = '500.html';
        }
      } else {
        setTimeout(function() {
          location.reload();
        }, 1000);
      }
    }).catch(error => {
      console.error('Error:', error);
    });
  }
  document.getElementById('uploadForm').addEventListener('submit', function(event) {
    event.preventDefault();
    const fileInput = document.getElementById('fileInput');
    const file = fileInput.files[0];
    const reader = new FileReader();
    reader.readAsArrayBuffer(file);
    reader.onloadend = function() {
      const arrayBuffer = reader.result;
      const uint8Array = new Uint8Array(arrayBuffer);
      fetch('/uploads', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
          'X-File-Name': file.name
        },
        body: uint8Array,
        redirect: 'manual'
      }).then(response => {
        console.log(response.status);
        if (!response.ok) {
          if (response.status === 400) {
            console.log('400 crap piles from 01 delivered');
            window.location.href = '400.html';
          }
          else if (response.status === 403) {
            console.log('403 crap piles from 01 delivered');
            window.location.href = '403.html';
          }
          else if (response.status === 404) {
            console.log('404 crap piles from 01 delivered');
            window.location.href = '404.html';
          }
          else if (response.status === 405) {
            console.log('405 Method Not Allowed');
            window.location.href = '405.html';
          }
          else if (response.status === 413) {
            console.log('413 crap piles from 01 delivered');
            window.location.href = '413.html';
          }
          else {
            console.log('500 crap piles from 01 delivered');
            window.location.href = '500.html';
          }
        } else {
          setTimeout(function() {
            location.reload();
          }, 1000);
        }
      }).catch(error => {
        console.error('Error:', error);
      });
    };
  });
  </script>
  "#;
  
  html.push_str(&script);
  
  html
}


/// when some file requested from uploads folder,
/// 
/// this function manages processing of this request.
/// 
/// Managed separately, because the uploads folder is dynamic. Not safe to use.
pub fn handle_uploads_get_uploaded_file(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: PathBuf,
  file_path: String,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  // todo: refactor path check to os separator instead of hardcoding of / ... probably
  
  // analyze path. if path is directory, then return default file, according to server config
  let mut path = request.uri().path();
  // cut first slash
  if path.starts_with("/"){ path = &path[1..]; }
  
  let absolute_path = zero_path_buf.join("uploads").join(file_path);
  
  // check if path is directory, then return default file as task requires
  if path.ends_with("/") || absolute_path.is_dir() {
    return response_default_static_file(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    );
  } else if !absolute_path.is_file() {
    
    eprintln!("ERROR:\n-----------------------------------\nuploads absolute_path IS NOT A FILE \n-----------------------------------"); // todo: remove dev print
    
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      StatusCode::NOT_FOUND,
    )
  } // check if file exists or return 404
  
  
  let parts: Vec<&str> = path.split('/').collect();
  
  // only GET method allowed for this path. filtering happens above
  let allowed_methods = vec!["GET".to_string()];
  
  // check if method is allowed for this path or return 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    eprintln!("ERROR: method {} is not allowed for path {}", request_method_string, path);
    return custom_response_4xx(
      request,
      cookie_value,
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
        cookie_value,
        zero_path_buf,
        server_config
      )
    }
  };
  
  let mut response = match Response::builder()
  .status(StatusCode::OK)
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
        server_config)
      }
    };
    
    // get file mime type using mime_guess, or use the text/plain
    let mime_type = match mime_guess::from_path(absolute_path.clone()).first(){
      Some(v) => v.to_string(),
      None => "text/plain".to_string(),
    };
    // println!("\n-------\n\nmime_type {}\n\n----------\n", mime_type); //todo: remove dev print
    
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
  
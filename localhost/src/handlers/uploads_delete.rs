use std::{path::PathBuf, fs};

use http::Request;


pub fn delete_the_file_from_uploads_folder(
  request: &Request<Vec<u8>>,
  absolute_path: &PathBuf
) {
  
  println!("=== INSIDE delete_the_file_from_uploads_folder");

  let body = std::str::from_utf8(&request.body()).unwrap();
  let params: Vec<&str> = body.split('&').collect();
  let mut file_name = "";
  for param in params {
    let key_value: Vec<&str> = param.split('=').collect();
    if key_value[0] == "file" {
      file_name = key_value[1];
      break;
    }
  }
  
  let file_path = absolute_path.join(file_name);
  // check if file exists, then delete
  if file_path.is_file() {
    fs::remove_file(file_path).unwrap();
    // wait while file system deletes the file
    // std::thread::sleep(std::time::Duration::from_millis(200));
  } else {
    eprintln!("ERROR: no file \"{:?}\" detected", file_path);
  }
}

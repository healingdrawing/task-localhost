use std::{path::PathBuf, fs};

use http::Request;

use crate::stream::errors::{ERROR_200_OK, ERROR_400_BAD_REQUEST};
use crate::stream::errors::ERROR_500_INTERNAL_SERVER_ERROR;

pub fn delete_the_file_from_uploads_folder(
  request: &Request<Vec<u8>>,
  absolute_path: &PathBuf
) -> String {
  
  let body = match std::str::from_utf8(&request.body()){
    Ok(v) => v,
    Err(_e) => return ERROR_400_BAD_REQUEST.to_string(),
  };
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
    match fs::remove_file(file_path){
      Ok(_v) => (),
      Err(_e) => return ERROR_500_INTERNAL_SERVER_ERROR.to_string(),
    };
    // wait while file system deletes the file
    // std::thread::sleep(std::time::Duration::from_millis(1000));
  } else {
    eprintln!("ERROR: No file \"{:?}\" detected", file_path);
  }

  ERROR_200_OK.to_string()
}

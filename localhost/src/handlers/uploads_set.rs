use std::{path::PathBuf, fs};

use http::Request;

use sanitise_file_name::sanitise;

use crate::stream::errors::{ERROR_200_OK, ERROR_500_INTERNAL_SERVER_ERROR};
use crate::stream::errors:: ERROR_400_HEADERS_FAILED_TO_PARSE;
use crate::stream::errors::ERROR_400_HEADERS_KEY_NOT_FOUND;

pub fn upload_the_file_into_uploads_folder(request: &Request<Vec<u8>>, absolute_path: &PathBuf) -> String {
  
  let file_content = &request.body()[..];
  
  let header_value = match request.headers().get("X-File-Name") {
    Some(v) => v,
    None => return ERROR_400_HEADERS_KEY_NOT_FOUND.to_string(),
  };
  let file_name = match header_value.to_str() {
    Ok(v) => v,
    Err(_e) => return ERROR_400_HEADERS_FAILED_TO_PARSE.to_string(),
  };

  
  // Sanitize the file name
  let sanitised_file_name = sanitise( file_name );
  let sanitised_file_name = sanitised_file_name.replace(" ", "_");
  // Remove double underscores
  let sanitised_file_name = sanitised_file_name.replace("__", "_");
  
  let file_path = absolute_path.join(sanitised_file_name);
  match fs::write(file_path, file_content){
    Ok(_v) => (),
    Err(_e) => return ERROR_500_INTERNAL_SERVER_ERROR.to_string(),
  };
  
  ERROR_200_OK.to_string()
}

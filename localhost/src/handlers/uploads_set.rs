use std::{path::PathBuf, fs};

use http::Request;

use sanitise_file_name::sanitise;

pub fn upload_the_file_into_uploads_folder(request: &Request<Vec<u8>>, absolute_path: &PathBuf) {
 
 let file_content = &request.body()[..];
 let file_name = request.headers().get("X-File-Name").unwrap().to_str().unwrap();

 // Sanitize the file name
 let sanitised_file_name = sanitise( file_name );
 let sanitised_file_name = sanitised_file_name.replace(" ", "_");
 // Remove double underscores
 let sanitised_file_name = sanitised_file_name.replace("__", "_");

 let file_path = absolute_path.join(sanitised_file_name);
 fs::write(file_path, file_content).unwrap();
}

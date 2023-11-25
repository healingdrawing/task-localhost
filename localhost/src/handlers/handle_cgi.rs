use std::process::Command;

use http::{Request, Response};

use crate::server::ServerConfig;


pub fn handle_cgi_request(
  check_file_path: String,
  script_file_name: String,
  request: Request<Vec<u8>>,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  // run python script , and check the path is file,folder or not exist/wrong path

  // Set the system PATH_INFO or send request path_info into python3 script as argument
  let output = Command::new("python3")
  .arg("/path/to/your/cgi/useless.py")
  .arg(check_file_path)
  .output()
  .expect("Failed to execute command");

// Print the result from the Python3 script
println!("{}", std::str::from_utf8(&output.stdout).unwrap());

// Read the result inside Rust code
let result = std::str::from_utf8(&output.stdout).unwrap();

// Return the response or write to the stream
let body = format!("Hello from Rust and Python3: {}", result);
  let mut response = Response::new(body.into());
  
  response
  
}
use std::{process::Command, path::PathBuf};

use http::{Request, Response};

use crate::server::ServerConfig;

/// run python script , and check the path is file,folder or not exist/wrong path
/// 
/// unsafe potentially , because you can pass any path to the script using
/// 
/// cgi/useless.py//some/path/here. but in exact this case allow only to check
pub fn handle_cgi_request(
  zero_path: String,
  script_file_name: String,
  check_file_path: String,
  request: Request<Vec<u8>>,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  println!("\n\nhandle_cgi_request: check_file_path: {:?}", check_file_path); //todo: remove dev print
  
  println!("zero_path: {:?}", zero_path); //todo: remove dev print

  let script_path = "cgi/".to_owned() + &script_file_name;
  println!("script_path: {:?}", script_path); //todo: remove dev print

  // Set the system PATH_INFO or send request path_info into python3 script as argument
  let output = Command::new("python3")
  .arg(script_path)
  .arg(check_file_path)
  .output();

  let result = match &output{
    Ok(v) => match std::str::from_utf8(&v.stdout){
      Ok(v) => v,
      Err(e) => {
        let error_message = "Failed to convert cgi output to str. ".to_owned() + &e.to_string();
        Box::leak(error_message.into_boxed_str())
      }
    },
    Err(e) => {
      let error_message = "Failed to get cgi output. ".to_owned() + &e.to_string();
      Box::leak(error_message.into_boxed_str())
    } // new puzzle. instead of return error as string(instead of coding), fuck your brain for hours , with borrowing/binding/lifetime etc, because it is just can not do this naturally. Rust is crap
  };
  
  // .expect("Failed to execute command");

  // // Print the result from the Python3 script
  // println!("Result is:{}", std::str::from_utf8(&output.stdout).unwrap());
  
  // // Read the result inside Rust code
  // let result = std::str::from_utf8(&output.stdout).unwrap();
  
  // write to the stream
  let body = format!("Hello from Rust and Python3: {}\n\n", result);
  let mut response = Response::new(body.as_bytes().to_vec());
  response.headers_mut().insert("Content-Type", "text/plain".parse().unwrap());

  response
  
}

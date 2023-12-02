use core::panic;
use std::error::Error;

use walkdir::WalkDir;

use crate::server::ServerConfig;

/// add static files to server configs routes, with method GET
pub fn add_static_files_to_server_configs(server_configs: &mut Vec<ServerConfig>) -> Result<(), Box<dyn Error>>{
   // static files relative path prefix
  let static_files_root = "static/".to_owned();
  for server_config in server_configs{
    let static_files_prefix = static_files_root.to_owned() + &server_config.static_files_prefix.to_owned();
    // get the routes to add static files to... css images etc
    let routes = &mut server_config.routes;
    
    // walk through static files folder recursively
    for entry in WalkDir::new(&static_files_prefix).into_iter().filter_map(|e| e.ok()) {
      // get the file path
      let file_path = entry.path();
      // check if it is a file
      if !file_path.is_file(){ continue; }

      // relative path to static files folder
      let relative_file_path = match file_path.strip_prefix(&static_files_prefix){
        Ok(v) => v,
        Err(e) => panic!("Failed to strip prefix: {} from file path: {} | {}", static_files_prefix, file_path.display(), e),
      };

      println!("add \"{}\"", relative_file_path.to_string_lossy().trim_start_matches(&static_files_prefix));

      // add the route to the server config, with method GET
      let key = match relative_file_path.to_str(){
        Some(v) => v.to_owned(),
        None => panic!("Failed to convert file path to str. Static file path: {}", relative_file_path.display()),
      };
    
      let value = vec!["GET".to_owned()];

      routes.insert(key, value);

      // get the file name
      let file_name = match relative_file_path.file_name(){
        Some(v) => v,
        None => continue,
      };
      // get the file name as string
      let file_name = match file_name.to_str(){
        Some(v) => v,
        None => continue,
      };

      // println!("static file: {}", file_name);

    }

  }
  
  return Ok(())
}
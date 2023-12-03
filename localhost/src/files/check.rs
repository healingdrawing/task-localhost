use std::{path::Path, f32::consts::E};

use crate::server::core::ServerConfig;

//ok it works
pub fn dummy_check_file_path(){
  let path = Path::new("debug.txt");
   if path.exists() {
       println!("File exists! {:?}", path);
   } else {
       println!("File does not exist! {:?}", path);
   }
}

//todo: implement the function to check the file by relative path in time of server configuration check, before run the server. Should be used to prevent run without required files described in config etc. Later after server run, if some files will be removed, server , according to task requirements must continue to work. To implement this, the emergency server error 500 page must be hardcoded into executable, and the server must be able to serve it without any files.

pub fn file_exists(path: &str) -> bool{
  let path = Path::new(path);
    if path.exists() {
        // println!("File exists! {:?}", path); //todo: remove this dev print
        return true;
    } else {
        println!("File does not exist! {:?}", path);
        return false;
    }
}

const ERROR_PAGES: [&str; 6] = ["400.html", "403.html", "404.html", "405.html", "413.html", "500.html"];

/// check relative paths. The parent level is executable folder.
/// 
/// Just a minimum files check, to prevent server run without files required by task
pub fn all_files_exists(server_configs: &Vec<ServerConfig>) -> bool{
  
  // check cgi script required by task
  if !file_exists("cgi/useless.py"){
    println!("cgi/useless.py does not exist");
    return false
  }

  // check custom error pages required by task
  for server_config in server_configs{
    let error_prefix =
    "static/".to_owned()+&server_config.error_pages_prefix; // error pages path prefix
    for file_name in ERROR_PAGES{
      if !file_exists( &(error_prefix.to_owned() + "/" + file_name)){
        println!("Error page {} does not exist", file_name);
        return false
      }
    }

    // check default file required by task
    let static_prefix =
    "static/".to_owned()+&server_config.static_files_prefix; // static files path prefix
    if !file_exists( &(static_prefix.to_owned() + "/" + &server_config.default_file)){
      println!("Default file {} does not exist", &server_config.default_file);
      return false
    }

  }

  true
}
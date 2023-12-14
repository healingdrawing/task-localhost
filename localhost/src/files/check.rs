use std::path::Path;
use sanitise_file_name::sanitise;

use crate::server::core::ServerConfig;


/// relative path from executable folder/level
/// 
/// named file_exists , but it checks path, not only files. But used only for files.
pub fn file_exists(path: &str) -> bool{
  let path = Path::new(path);
    if path.exists() {
        // println!("File exists! {:?}", path); //todo: remove this dev print
        return true;
    } else {
        eprintln!("ERROR: File does not exist! {:?}", path);
        return false;
    }
}

pub const ERROR_PAGES: [&str; 6] = ["400.html", "403.html", "404.html", "405.html", "413.html", "500.html"];

/// check relative paths. The parent level is executable folder.
/// 
/// Just a minimum files check, to prevent server run without files required by task
pub async fn all_files_exists(server_configs: &Vec<ServerConfig>) -> bool{
  
  // check cgi script required by task
  if !file_exists("cgi/useless.py"){
    eprintln!("ERROR: Path cgi/useless.py does not exist");
    return false
  }

  // check custom error pages required by task
  for server_config in server_configs{
    let error_prefix =
    "static/".to_owned()+&server_config.error_pages_prefix; // error pages path prefix
    for file_name in ERROR_PAGES{
      if !file_exists( &(error_prefix.to_owned() + "/" + file_name)){
        eprintln!("ERROR: Error page {} does not exist", file_name);
        return false
      }
    }

    // check default file required by task
    let static_prefix =
    "static/".to_owned()+&server_config.static_files_prefix; // static files path prefix
    if !file_exists( &(static_prefix.to_owned() + "/" + &server_config.default_file)){
      eprintln!("ERROR: Default file {} does not exist", &server_config.default_file);
      return false
    }

  }

  true
}

/// check the path file ends with error page name. Only implemented error pages checked.
pub fn is_implemented_error_page(path: &str) -> bool{
  for error_page in ERROR_PAGES{
    if path.ends_with(error_page){ return true }
  }
  false
}

/// sanitise + replace spaces to underscores + replace double underscores to single underscore
pub fn sanitise_file_name(file_name: &str) -> String{
  sanitise( file_name ).replace(" ", "_").replace("__", "_")
}
pub fn bad_file_name(file_name: &str) -> bool{ sanitise_file_name(file_name) != file_name }
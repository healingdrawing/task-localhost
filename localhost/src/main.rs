mod debug;
use debug:: {
  try_recreate_file_according_to_value_of_debug_boolean,
  create_something_in_uploads_folder
};

pub mod server{
  pub mod find;
  pub mod core;
  pub mod flow;
}
use server::core::ServerConfig;
use server::flow::run;

pub mod stream{
  pub mod read;
  pub mod parse;
  pub mod write_;
  pub mod write_error;
  pub mod errors;
}

pub mod handlers{
  pub mod handle_;
  pub mod handle_cgi;
  pub mod handle_uploads;
  pub mod handle_all;
  pub mod response_;
  pub mod response_500;
  pub mod response_4xx;
  pub mod uploads_get;
  pub mod uploads_delete;
}

pub mod files{
  pub mod add_static;
  pub mod check;
}

use std::{env, path::PathBuf};
use std::error::Error;
use config::{File, FileFormat};

use crate::files::add_static::add_static_files_to_server_configs;
use crate::files::check::all_files_exists;

/// get exe path and cut off exe name. to manage the config, cgi, etc folders
pub fn get_zero_path() -> Result<PathBuf, Box<dyn Error>>{
  let mut exe_path = match env::current_exe(){
    Ok(v) => v,
    Err(e) => return Err(format!("Failed to get current exe path: {}", e).into()),
  };
  
  match exe_path.pop(){
    true => {},
    false => return Err("Failed to pop current exe path".into()),
  }; // Remove the executable name from the path
  
  Ok(exe_path)
}

fn main() {
  println!("Hello, world!");
  
  match create_something_in_uploads_folder(){
    Ok(_) => println!("\"something\" created in uploads folder"),
    Err(e) => panic!("\"something\" creation failed: {}", e),
  };

  match try_recreate_file_according_to_value_of_debug_boolean(){
    Ok(_) => println!("debug file recreated"),
    Err(e) => panic!("debug file recreation failed: {}", e),
  }
  
  // manage settings, cgi and so on
  let zero_path_buf = match get_zero_path(){
    Ok(v) => v,
    Err(e) => panic!("Failed to manage current exe path: {}", e),
  };
  let zero_path:String = match zero_path_buf.to_str(){
    Some(v) => v,
    None => panic!("Failed to convert zero_path to str"),
  }
  .to_string();
  // set PATH_INFO here to check inside cgi, as task requires.
  // it is dumb(python3 can manage it + no need reload server after changes in script).
  // Since we use python3, anyways the process will be slowdowned by this,
  // so no reason to do this using rust. Easier just send it into script as argument.
  env::set_var("PATH_INFO", zero_path.clone()); //todo: can be unsafe
  
  let mut config_path = zero_path_buf.clone();
  config_path.push("settings"); // Add the configuration file name to the path
  
  let mut settings = config::Config::builder();
  let config_path_str = match config_path.to_str(){
    Some(v) => v,
    None => panic!("Failed to convert config_path to str"),
  };
  
  settings = settings.add_source(
    File::new(config_path_str , FileFormat::Toml)
  );
  
  let settings = settings.build();
  
  match settings {
    Ok(config) => {
      let server_configs: Result<Vec<ServerConfig>, _> = config.get("servers");
      match server_configs {
        Ok(mut server_configs) =>{ // configuration read successfully
          // check if at least one server config exists
          if server_configs.len() == 0{ panic!("No correct server configs"); }

          // clean up server_configs
          for sc in server_configs.iter_mut(){sc.check()}
          // check if all required by task files exists
          if !all_files_exists(&server_configs){
            panic!("Not all required for server config files exists");
          };
          
          //todo: call/implement add static files to server_configs
          match add_static_files_to_server_configs(&mut server_configs){
            Ok(_) => println!("static files added to server configs, with method GET"),
            Err(e) => panic!("Failed to add static files to server configs: {}", e),
          };
          
          println!("{:#?}", server_configs); //todo: remove this dev print
          run( zero_path_buf ,server_configs);
        },
        Err(e) => eprintln!("Failed to convert settings into Vec<ServerConfig>: {}", e),
      }
    }
    Err(e) => eprintln!("Failed to build settings: {}", e),
  }
}

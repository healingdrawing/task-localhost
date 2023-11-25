mod debug;
use debug:: try_recreate_file_according_to_value_of_debug_boolean;

mod server;
use server::{ServerConfig, run};

pub mod stream{
  pub mod read;
  pub mod parse;
  pub mod write;
}

pub mod handlers{
  pub mod handle_;
  pub mod handle_cgi;
}

use std::{env, path::PathBuf};
use std::error::Error;
use config::{ConfigError, File, FileFormat};

use std::process::{Command, Stdio};

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
  match try_recreate_file_according_to_value_of_debug_boolean(){
    Ok(_) => println!("debug file recreated"),
    Err(e) => println!("debug file recreation failed: {}", e),
  }
  
  let mut config_path = match get_zero_path(){
    Ok(v) => v,
    Err(e) => panic!("Failed to manage current exe path: {}", e),
  };
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

          for sc in server_configs.iter_mut(){sc.check()}

          println!("{:#?}", server_configs); //todo: remove this dev print
          run(server_configs);//todo: looks like need send exe_path to run() to manage the config, cgi, etc folders
        },
        Err(e) => eprintln!("Failed to convert settings into Vec<ServerConfig>: {}", e),
      }
    }
    Err(e) => eprintln!("Failed to build settings: {}", e),
  }
}

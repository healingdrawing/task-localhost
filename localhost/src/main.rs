mod debug;
use debug:: try_recreate_file_according_to_value_of_debug_boolean;

mod server;
use server::{ServerConfig, run};

pub mod stream{
  pub mod read;
  pub mod parse;
}

pub mod handlers{
  pub mod handle_;
}

use std::env;
use config::{ConfigError, File, FileFormat};

use std::process::{Command, Stdio};

fn main() {
  println!("Hello, world!");
  match try_recreate_file_according_to_value_of_debug_boolean(){
    Ok(_) => println!("debug file recreated"),
    Err(e) => println!("debug file recreation failed: {}", e),
  }
  
  let mut config_path = match env::current_exe(){
    Ok(v) => v,
    Err(e) => panic!("Failed to get current exe path: {}", e),
  };
  config_path.pop(); // Remove the executable name from the path
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
          run(server_configs);
        },
        Err(e) => eprintln!("Failed to convert settings into Vec<ServerConfig>: {}", e),
      }
    }
    Err(e) => eprintln!("Failed to build settings: {}", e),
  }
}

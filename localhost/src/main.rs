mod debug;
use debug:: try_recreate_file_according_to_value_of_debug_boolean;

mod server;
use server::{ServerConfig, run};

pub mod stream{
  pub mod read;
  pub mod parse;
  pub mod write_;
  pub mod write_error;
}

pub mod handlers{
  pub mod handle_;
  pub mod handle_cgi;
}

pub mod files{
  pub mod check;
}
use files::check::dummy_check_file_path;

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

  dummy_check_file_path(); //todo: remove this dev print


  match try_recreate_file_according_to_value_of_debug_boolean(){
    Ok(_) => println!("debug file recreated"),
    Err(e) => println!("debug file recreation failed: {}", e),
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

          for sc in server_configs.iter_mut(){sc.check()}

          println!("{:#?}", server_configs); //todo: remove this dev print
          run( zero_path ,server_configs);//todo: looks like need send exe_path to run() to manage the config, cgi, etc folders
        },
        Err(e) => eprintln!("Failed to convert settings into Vec<ServerConfig>: {}", e),
      }
    }
    Err(e) => eprintln!("Failed to build settings: {}", e),
  }
}

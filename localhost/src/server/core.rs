use mio::Token;
use mio::net::TcpListener;
use serde::Deserialize;
use uuid::Uuid;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use std::time::SystemTime;


#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub server_name: String,
  pub ports: Vec<String>,
  pub server_address: String,
  pub client_body_size: usize,
  pub static_files_prefix: String,
  pub default_file: String,
  pub error_pages_prefix: String,
  pub uploads_methods: Vec<String>,
  pub routes: HashMap<String, Vec<String>>,
}

impl ServerConfig {
  /// drop out all ports not in 0..65535 range, also drop out all repeating of ports
  pub fn check(&mut self){
    self.check_ports();
  }
  
  /// drop out all ports not in 0..65535 range, also drop out all repeating of ports
  fn check_ports(&mut self){
    let old_ports = self.ports.clone();
    let mut ports: HashSet<String> = HashSet::new();
    for port in self.ports.iter(){
      let port: u16 = match port.parse(){
        Ok(v) => v,
        Err(e) =>{
          eprintln!("Config \"{}\" Failed to parse port: {} into u16", self.server_name, e);
          continue;
        }
      };
      ports.insert(port.to_string());
    }
    self.ports = ports.into_iter().collect();
    if self.ports.len() != old_ports.len(){
      eprintln!("=== Config \"{}\" ports changed\nfrom {:?}\n  to {:?}", self.server_name, old_ports, self.ports);
    }
    
  }
  
}

/// get list of unique ports from server_configs, to use listen 0.0.0.0:port
/// 
/// and manage pseudo servers, because the task requires redirection
/// 
/// if no declared server name in request, so we need to use "default" server
pub fn get_usize_unique_ports(server_configs: &Vec<ServerConfig>) -> Result<Vec<usize>, Box<dyn Error>>{
  let mut ports: HashSet<usize> = HashSet::new();
  for server_config in server_configs.iter(){
    for port in server_config.ports.iter(){
      let port: u16 = match port.parse(){
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to parse port: {} into u16: {}", port, e).into()),
      };
      ports.insert(port as usize);
    }
  }
  let ports: Vec<usize> = ports.into_iter().collect();
  
  if ports.len() < 1 {
    return Err("Not enough correct ports declared in \"settings\" file".into());
  }
  
  Ok(ports)
  
}

#[derive(Debug)]
pub struct Server {
  pub listener: TcpListener,
  pub token: Token,
  pub cookies: HashMap<String, Cookie>,
  pub cookies_check_time: SystemTime,
}

#[derive(Debug)]
pub struct Cookie {
  pub name: String,
  pub value: String,
  pub expiration: SystemTime,
}

impl Server {
  pub fn set_cookie(&mut self, name: String, value: String, life_time: Duration){
    let expiration = SystemTime::now() + life_time;
    self.cookies.insert(
      name.clone(),
      Cookie {
        name,
        value,
        expiration
      }
    );
  }
  
  pub fn get_cookie(&self, name: &str) -> Option<&Cookie>{
    self.cookies.get(name)
  }
  
  /// get cookie by name, if cookie not found, then generate new cookie for one minute
  /// 
  /// return header value for cookie as string "{}={}; Expires={}; HttpOnly; Path=/" to send in response
  pub fn send_cookie(&mut self, name: String) -> String {
    if let Some(cookie) = self.cookies.get(&name){
      let name = cookie.name.clone();
      let value = cookie.value.clone();
      let expiration = cookie.expiration.clone();
      let expires = match
      expiration.duration_since(SystemTime::UNIX_EPOCH){
        Ok(v) => v,
        Err(e) => {
          eprintln!("ERROR: Failed to get duration_since for cookie name {}: {}", name, e);
          Duration::new(0, 0)
        }
      }
      .as_secs();
      
      let cookie = format!(
        "{}={}; Expires={}; HttpOnly; Path=/", name, value, expires
      );
      
      return cookie
      
    } else { // if cookie not found, then generate new cookie for one minute
      let name = Uuid::new_v4().to_string();
      let value = Uuid::new_v4().to_string();
      self.set_cookie( name.clone(), value.clone(), Duration::from_secs(60) );
      
      return format!(
        "{}={}; Expires={}; HttpOnly; Path=/\r\n", name, value, 60
      );

    }

  }
  
  /// if cookie expired, then remove it from cookies, and return true,
  /// 
  /// if cookie not found,then return true, as signal, to generate new cookie,
  /// 
  /// else return false
  pub fn is_cookie_expired(&mut self, name: &str) -> bool {
    let now = SystemTime::now();
    if let Some(cookie) = self.cookies.get(name){
      if cookie.expiration < now{
        self.cookies.remove(name);
        return true
      }
    } else {
      return true
    }
    false
  }
  
  /// remove all expired cookies. Used with timeout 60 sec, to not check every request
  pub fn check_expired_cookies(&mut self){
    let now = SystemTime::now();
    if now > self.cookies_check_time {
      // collect all expired cookies
      let mut expired_cookies = Vec::new();
      for (name, cookie) in self.cookies.iter(){
        if cookie.expiration < now {
          expired_cookies.push(name.clone());
          println!("expired cookie: {:?}", cookie); //todo: remove dev print
        }
      }
      // remove all expired cookies
      for name in expired_cookies.iter(){
        self.cookies.remove(name);
      }
    }
    // set next check time, one minute from now
    self.cookies_check_time = now + Duration::from_secs(60);
    
  }

}
use std::time::{SystemTime, Duration};
use http::Request;
use uuid::Uuid;

use super::core::Server;



#[derive(Debug)]
pub struct Cookie {
  pub name: String,
  pub value: String,
  pub expiration: SystemTime,
}

impl Server {

  pub fn generate_unique_cookie_and_return_value(&mut self) -> Cookie {
    let name = Uuid::new_v4().to_string();
    let value = Uuid::new_v4().to_string();
    let cookie = self.set_cookie( name.clone(), value.clone(), Duration::from_secs(60) );
    cookie
  }

  pub fn set_cookie(&mut self, name: String, value: String, life_time: Duration) -> Cookie {
    let expiration = SystemTime::now() + life_time;
    let cookie = Cookie { name, value, expiration };
    self.cookies.insert( name.clone(), cookie );
    cookie
  }
  
  pub fn get_cookie(&self, name: &str) -> Option<&Cookie>{ self.cookies.get(name) }

  pub fn extract_cookies_from_request(
    &mut self,
    request: &Request<Vec<u8>>
  ) -> String {
    // have value
    let mut cookie_names = Vec::new();
    // does not have value
    let flags = String::new();
    // expiration time
    let expiration_string = String::new();
    let mut is_expired = false;
    
   
    let cookie_header = match request.headers().get("Cookie"){
      Some(v) => v,
      None =>{
        println!("no cookie header, new one will be generated"); //todo: remove dev print
        
        return self.send_cookie(Uuid::new_v4().to_string())
      }
    };

    let cookie_header = match cookie_header.to_str(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to get cookie_header.to_str: {}", e);
        return self.send_cookie(Uuid::new_v4().to_string())
      }
    };

    // split cookie header by "; " to get all cookie parts, like "name=value" or "name" for flags
    let cookie_parts:Vec<&str> = cookie_header.split("; ").collect();
    
    // check all cookie parts and find Expired name to check it
    // if cookie expired, then do not write it in the server.cookies and return new cookie
    for cookie_part in cookie_parts.iter(){
      let cookie_part: Vec<&str> = cookie_part.splitn(2, '=').collect();
      let part_name = cookie_part[0];

      if part_name == "Expired"{
        let part_value = cookie_part[1];
        let expiration = match part_value.parse::<u64>(){
          Ok(v) => v,
          Err(e) => {
            eprintln!("ERROR: Failed to parse cookie expiration: {}", e);
            return self.send_cookie(Uuid::new_v4().to_string())
          }
        };

      }

    }
   
    if cookie_names.is_empty() {
    let name = Uuid::new_v4().to_string();
    let expiration = SystemTime::now() + Duration::from_secs(60);
    let cookie = Cookie { name: name.clone(), value: "".to_string(), expiration };
    self.cookies.insert(name.clone(), cookie);
    cookie_names.push(name);
    }
   
    cookie_names
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
        "{}={}; Expires={}; Path=/", name, value, expires
      );
      
      return cookie
      
    } else { // if cookie not found, then generate new cookie for one minute
      let name = Uuid::new_v4().to_string();
      let value = Uuid::new_v4().to_string();
      self.set_cookie( name.clone(), value.clone(), Duration::from_secs(60) );
      
      return format!(
        "{}={}; Expires={}; Path=/", name, value, 60
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
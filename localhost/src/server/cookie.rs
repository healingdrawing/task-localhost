use std::time::{SystemTime, Duration};
use http::Request;
use uuid::Uuid;

use crate::server::core::Server;



#[derive(Debug, Clone)]
pub struct Cookie {
  pub name: String,
  pub value: String,
  /// expiration time in seconds from UNIX_EPOCH
  pub expires: u64,
}

impl Server {
  
  fn generate_unique_cookie_and_return(&mut self) -> Cookie {
    let name = Uuid::new_v4().to_string();
    let value = Uuid::new_v4().to_string();
    let cookie = self.set_cookie( name.clone(), value.clone(), Duration::from_secs(60) );
    cookie
  }
  
  pub fn set_cookie(&mut self, name: String, value: String, life_time: Duration) -> Cookie {
    let expiration = SystemTime::now() + life_time;
    let expires = match
    expiration.duration_since(SystemTime::UNIX_EPOCH){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to get duration_since for cookie name {}: {}", name, e);
        Duration::new(0, 0)
      }
    }
    .as_secs();
    
    let cookie = Cookie { name: name.clone(), value, expires };
    self.cookies.insert( name.clone(), cookie.clone() );
    cookie
  }
  
  pub fn get_cookie(&self, name: &str) -> Option<&Cookie>{ self.cookies.get(name) }
  
  /// extract cookies from request, if cookie not found, then generate new cookie for one minute.
  /// 
  /// Also return bool. False if cookie recognized as bad, for bad request response
  pub fn extract_cookies_from_request_or_provide_new(
    &mut self,
    request: &Request<Vec<u8>>
  ) -> (String, bool) {
    
    let cookie_header = match request.headers().get("Cookie"){
      Some(v) => v,
      None =>{ // no cookie header, new cookie will be generated
        let cookie = self.generate_unique_cookie_and_return();
        return (self.send_cookie(cookie.name), true)
      }
    };
    
    let cookie_header = match cookie_header.to_str(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to get cookie_header.to_str: {}", e);
        let cookie = self.generate_unique_cookie_and_return();
        return (self.send_cookie(cookie.name), false)
      }
    }
    .trim();
    
    // split cookie header by "; " to get all cookie parts, like "name=value" or "name" for flags.
    let cookie_parts:Vec<&str> = cookie_header.split("; ").collect();
    let cookie_parts:Vec<&str> = cookie_parts.iter().map(|v| v.trim()).collect();
    
    // check all cookie parts and find Expired name to check it
    // if cookie expired, then do not write it in the server.cookies and return new cookie
    let mut expires_not_found = true;
    let mut global_expiration: u64 = 0; // use it for write parts in server.cookies
    for cookie_part in cookie_parts.iter(){
      let cookie_part: Vec<&str> = cookie_part.splitn(2, '=').collect();
      let part_name = cookie_part[0];
      
      if part_name == "Expires"{
        expires_not_found = false;
        let part_value = cookie_part[1];
        let expiration = match part_value.parse::<u64>(){
          Ok(v) => v,
          Err(e) => {
            eprintln!("ERROR: Failed to parse cookie expiration: {}", e);
            let cookie = self.generate_unique_cookie_and_return();
            return (self.send_cookie(cookie.name), false)
          }
        };
        // check if cookie expired
        let now = SystemTime::now();
        let expires = SystemTime::UNIX_EPOCH + Duration::from_secs(expiration);
        if expires < now{ // if cookie expired, then do not write it in the server.cookies and return new cookie
          let cookie = self.generate_unique_cookie_and_return();
          return (self.send_cookie(cookie.name), true)
        }
        
        // check if cookie is too long living. Server provide only one minute cookies
        let max_life_time = SystemTime::now() + Duration::from_secs(60);
        if expires > max_life_time{
          eprintln!("ERROR: Cookie life time is too long. Potential security risk");
          let cookie = self.generate_unique_cookie_and_return();
          return (self.send_cookie(cookie.name), false)
        }
        
        global_expiration = expiration;
        
      }
      
    }
    
    // if expiration not found, generate new for one minute
    if expires_not_found{
      // println!("ERROR: \"Expires\" not found. Potential security risk");
      let new_expires  = SystemTime::now() + Duration::from_secs(60);
      global_expiration = match
      new_expires.duration_since(SystemTime::UNIX_EPOCH){
        Ok(v) => v.as_secs(),
        Err(e) => {
          eprintln!("ERROR: Failed to get duration_since for cookie expiration: {}", e);
          0
        }
      }
    }
    
    if global_expiration > 0{
      // parse common cookie parts and write them in server.cookies.
      // All name=value pairs except "Expires", which managed above
      for cookie_part in cookie_parts.iter(){
        let cookie_part: Vec<&str> = cookie_part.splitn(2, '=').collect();
        if cookie_part.len() == 2{
          let part_name = cookie_part[0];
          if part_name != "Expires"{
            let value = cookie_part[1];
            // write in server.cookies
            let cookie = Cookie {
              name: part_name.to_string(),
              value: value.to_string(),
              expires: global_expiration 
            };
            self.cookies.insert( part_name.to_string(), cookie );
          }
        }
      }
    }
    // if cookie is correct, then return it as value for header
    (cookie_header.to_string(), true)
    
  }
  
  /// get cookie by name, if cookie not found, then generate new cookie for one minute
  /// 
  /// return header value for cookie as string "{}={}; Expires={}; HttpOnly; Path=/" to send in response
  pub fn send_cookie(&mut self, name: String) -> String {
    if let Some(cookie) = self.cookies.get(&name){
      let name = cookie.name.clone();
      let value = cookie.value.clone();
      let expires = cookie.expires.clone();
      let cookie = format!(
        "{}={}; Expires={}; HttpOnly; Path=/", name, value, expires
      );
      return cookie
      
    } else { // if cookie not found, then generate new cookie for one minute
      let cookie = self.generate_unique_cookie_and_return();
      return format!(
        "{}={}; Expires={}; HttpOnly; Path=/", cookie.name, cookie.value, cookie.expires
      );
      
    }
    
  }
  
  /// if cookie expired, then remove it from cookies, and return true,
  /// 
  /// if cookie not found,then return true, as signal, to generate new cookie,
  /// 
  /// else return false
  // pub fn is_cookie_expired(&mut self, name: &str) -> bool {
  //   let now = SystemTime::now();
  //   if let Some(cookie) = self.cookies.get(name){
  //     let expiration = SystemTime::UNIX_EPOCH + Duration::from_secs(cookie.expires);
  //     if expiration < now{
  //       self.cookies.remove(name);
  //       return true
  //     }
  //   } else {
  //     return true
  //   }
  //   false
  // }
  
  /// remove all expired cookies. Used with timeout 60 sec, to not check every request
  pub fn check_expired_cookies(&mut self){
    let now = SystemTime::now();
    if now > self.cookies_check_time {
      // collect all expired cookies
      let mut expired_cookies = Vec::new();
      for (name, cookie) in self.cookies.iter(){
        let expiration = SystemTime::UNIX_EPOCH + Duration::from_secs(cookie.expires);
        if expiration < now {
          expired_cookies.push(name.clone());
          println!("expired cookie: {:?}", cookie); //todo: remove dev print
        }
      }
      // remove all expired cookies
      for name in expired_cookies.iter(){ self.cookies.remove(name); }
    }
    // set next check time, one minute from now
    self.cookies_check_time = now + Duration::from_secs(60);
    
  }
  
}
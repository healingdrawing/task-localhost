use std::time::{SystemTime, Duration};
use http::Request;
use uuid::Uuid;

use crate::{server::core::Server, debug::append_to_file};



#[derive(Debug, Clone)]
pub struct Cookie {
  pub name: String,
  pub value: String,
  /// expiration time in seconds from UNIX_EPOCH
  pub expires: u64,
}

impl Cookie {
  async fn is_expired(&self) -> bool {
    let now = SystemTime::now();
    let expiration = SystemTime::UNIX_EPOCH + Duration::from_secs(self.expires);
    expiration < now
  }

  async fn to_string(&self) -> String {
    // format!( "{}={}; Expires={}; HttpOnly; Path=/", self.name, self.value, self.expires )
    format!( "{}={}", self.name, self.value )
  }
}

impl Server {
  
  async fn generate_unique_cookie_and_return(&mut self) -> Cookie {
    let name = Uuid::new_v4().to_string();
    let value = Uuid::new_v4().to_string();
    let cookie = self.set_cookie( name.clone(), value.clone(), Duration::from_secs(60) ).await;
    cookie
  }
  
  async fn set_cookie(&mut self, name: String, value: String, life_time: Duration) -> Cookie {
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

    append_to_file(
      &format!( "===\n self.cookies before insert:\n{:?}\n===", self.cookies )
    ).await;

    self.cookies.insert( name.clone(), cookie.clone() );

    append_to_file(
      &format!( "===\n self.cookies after insert:\n{:?}\n===", self.cookies )
    ).await;


    cookie
  }
  
  pub async fn get_cookie(&self, name: &str) -> Option<&Cookie>{ self.cookies.get(name) }
  
  /// extract cookies from request, if cookie not found, then generate new cookie for one minute.
  /// 
  /// Also return bool. False if cookie recognized as bad, for bad request response
  pub async fn extract_cookies_from_request_or_provide_new(
    &mut self,
    request: &Request<Vec<u8>>
  ) -> (String, bool) {

    append_to_file("EXTRACT COOKIES FROM REQUEST OR PROVIDE NEW").await;
    let cookie_header_value = match request.headers().get("Cookie"){
      Some(v) =>{
        append_to_file(&format!( "Cookie header value: {:?}", v )).await;
        v
      },
      None =>{ // no cookie header, new cookie will be generated
        append_to_file("No \"Cookie\" header").await;
        let cookie = self.generate_unique_cookie_and_return().await;
        append_to_file(&format!( "New cookie: {:?}", cookie )).await;
        return (self.send_cookie(cookie.name).await, true)
      }
    };
    
    let cookie_header_value_str = match cookie_header_value.to_str(){
      Ok(v) => v,
      Err(e) => {
        eprintln!("ERROR: Failed to get cookie_header.to_str: {}", e);
        let cookie = self.generate_unique_cookie_and_return().await;
        return (self.send_cookie(cookie.name).await, false)
      }
    }
    .trim();
    
    // split cookie header by "; " to get all cookie parts, like "name=value" or "name" for flags.
    let cookie_parts:Vec<&str> = cookie_header_value_str.split("; ").collect();
    let cookie_parts:Vec<&str> = cookie_parts.iter().map(|v| v.trim()).collect();
    
    append_to_file(
      &format!( "===\n incoming Cookie parts: {:?}\n===", cookie_parts )
    ).await;

    append_to_file(
      &format!( "===\n server.cookies: {:?}\n===", self.cookies )
    ).await;


    // check all cookie parts, try to find them in server.cookies
    // if cookie not found, then generate new cookie for one minute
    // if cookie found, then check if it expired, if yes, then remove it from server.cookies and generate new cookie for one minute and return it as value for header
    // if cookie not expired, then return it as value for header
    // if found more then one cookie in server.cookies, then generate new cookie for one minute and return it as value for header
    let mut cookie_found = false;
    let mut broken_cookie_found = false;
    let mut expired_cookie_found = false;
    let mut more_then_one_cookie_found = false;
    let mut found_cookie_name = String::new();

    for cookie_part in cookie_parts.iter(){
      let cookie_part: Vec<&str> = cookie_part.splitn(2, '=').collect();
      let part_name = cookie_part[0];

      if let Some(server_cookie) = self.cookies.get(part_name){
        if cookie_found { more_then_one_cookie_found = true; }
        cookie_found = true;
        
        // check if cookie is correct
        if cookie_part.len() == 2 {
          let part_value = cookie_part[1];
          if part_value != server_cookie.value{
            eprintln!("ERROR: Cookie value is not correct. Potential security risk");
            broken_cookie_found = true;
          } else if !more_then_one_cookie_found { // first cookie found, use it
            found_cookie_name = part_name.to_string();
          }
        }
        
        // check if server cookie with the same name is expired
        if server_cookie.is_expired().await{
          expired_cookie_found = true;
          self.cookies.remove(part_name);
        }

      }

    }

    if expired_cookie_found || !cookie_found
    {
      let cookie = self.generate_unique_cookie_and_return().await;
      return (self.send_cookie(cookie.name).await, true)
    } else if broken_cookie_found || more_then_one_cookie_found {
      let cookie = self.generate_unique_cookie_and_return().await;
      return (self.send_cookie(cookie.name).await, false)
    } else {
      return (self.send_cookie(found_cookie_name).await, true)
    }
    
  }
  
  /// get cookie by name, if cookie not found, then generate new cookie for one minute
  /// 
  /// return header value for cookie as string "{}={}; Expires={}; HttpOnly; Path=/" to send in response
  pub async fn send_cookie(&mut self, name: String) -> String {
    if let Some(cookie) = self.cookies.get(&name){
      return cookie.to_string().await;
    } else { // if cookie not found, then generate new cookie for one minute
      let cookie = self.generate_unique_cookie_and_return().await;
      return cookie.to_string().await;
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
  pub async fn check_expired_cookies(&mut self){
    let now = SystemTime::now();
    if now > self.cookies_check_time {
      // collect all expired cookies
      let mut expired_cookies = Vec::new();
      for (name, cookie) in self.cookies.iter(){
        let expiration = SystemTime::UNIX_EPOCH + Duration::from_secs(cookie.expires);
        if expiration < now {
          expired_cookies.push(name.clone());
          append_to_file(&format!( "EXPIRED COOKIE: {:?}", cookie )).await;
        }
      }
      // remove all expired cookies
      for name in expired_cookies.iter(){ self.cookies.remove(name); }
    }
    // set next check time, one minute from now
    self.cookies_check_time = now + Duration::from_secs(60);
    
  }
  
}
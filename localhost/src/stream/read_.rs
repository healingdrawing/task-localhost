use std::time::{Instant, Duration};
use std::io::{self};

use futures::AsyncReadExt;
use async_std::net::TcpStream;

use crate::server::core::ServerConfig;
use crate::server::find::server_config_from_headers_buffer_or_use_default;
use crate::stream::errors::{ERROR_400_HEADERS_READ_TIMEOUT, ERROR_400_HEADERS_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE, ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNK_READ_TIMEOUT, ERROR_400_BODY_CHUNK_READING_STREAM, ERROR_400_BODY_CHUNK_IS_BIGGER_THAN_CHUNK_SIZE, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH, ERROR_400_BODY_READ_TIMEOUT, ERROR_400_DIRTY_BODY_READ_TIMEOUT, ERROR_400_BODY_READING_STREAM, ERROR_413_BODY_SIZE_LIMIT};

/// Read from the stream until timeout or EOF
/// 
/// returns a tuple of two vectors: (headers_buffer, body_buffer)
pub async fn read_with_timeout(
  timeout: Duration,
  stream: &mut TcpStream,
  headers_buffer: &mut Vec<u8>,
  body_buffer: &mut Vec<u8>,
  server_configs: &Vec<ServerConfig>,
  global_error_string: &mut String,
) -> ServerConfig {
  println!("\nINSIDE read_with_timeout"); //todo: remove later
  
  // Start the timer
  let start_time = Instant::now();
  
  // Read from the stream until timeout or EOF
  let mut buf = [0; 1];
  
  // ------------------------------------
  // collect request headers section
  // ------------------------------------
  
  loop {
    // Check if the timeout has expired
    if start_time.elapsed() >= timeout {
      eprintln!("ERROR: Headers read timed out");
      *global_error_string = ERROR_400_HEADERS_READ_TIMEOUT.to_string();
      return server_configs[0].clone();
    }
    
    match stream.read(&mut buf).await {
      Ok(0) => {
        // EOF reached
        println!("read EOF reached");
        break;
      },
      Ok(n) => {
        // Successfully read n bytes from stream
        // println!("attempt to read {} bytes from stream", n);
        headers_buffer.extend_from_slice(&buf[..n]);
        // println!("after read headers buffer size: {}", headers_buffer.len());
        // println!("after read headers buffer: {:?}", headers_buffer);
        // println!("after read headers buffer to string: {:?}", String::from_utf8(headers_buffer.clone()));
        // Check if the end of the stream has been reached
        if n < buf.len() {
          println!("read EOF reached relatively, because buffer not full after read");
          break;
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Stream is not ready yet, try again later
        continue;
      },
      Err(e) => {
        // Other error occurred
        eprintln!("ERROR: Reading headers from stream: {}", e);
        *global_error_string = ERROR_400_HEADERS_READING_STREAM.to_string();
        return server_configs[0].clone();
      },
    }
    
    
    if headers_buffer.ends_with(b"\r\n\r\n") {
      break;
    }
  }
  
  // chose the server_config and check the server_config.client_body_size
  // response 413 error, if body is bigger.
  // Duplicated fragment of code, because of weird task requirements.
  
  let server_config = server_config_from_headers_buffer_or_use_default(
    headers_buffer,
    server_configs.clone()
  ).await;
  
  // check of the body length, according to server_config.client_body_size.
  let client_body_size = server_config.client_body_size;
  let mut body_size = 0_usize;
  
  // not nice
  let dirty_string = String::from_utf8_lossy(&headers_buffer);
  let is_chunked = 
  dirty_string.contains("Transfer-Encoding: chunked")
  || dirty_string.contains("Transfer-Encoding: Chunked")
  || dirty_string.contains("transfer-encoding: chunked")
  || dirty_string.contains("transfer-encoding: Chunked");
  
  // ------------------------------------
  // collect request body section
  // ------------------------------------
  
  /*

  if is_chunked {
    println!("THE REQUEST IS CHUNKED");
    
    let mut sum_chunk_size_buffer = Vec::new();
    
    loop { // since timeout implemented, skip the first chunk size line which is sum of all chunks
      // Check if the timeout has expired
      if start_time.elapsed() >= timeout {
        eprintln!("ERROR: Sum chunk size read timed out");
        *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT.to_string();
        return server_config;
      }
      
      // Read from the stream one byte at a time
      match stream.read(&mut buf).await {
        Ok(0) => { println!("read EOF reached"); break },
        Ok(n) => { // Successfully read n bytes from stream
          
          body_size += n;
          
          // Check if the body size is bigger than client_body_size
          if body_size > client_body_size {
            eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
            *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
            return server_config;
          }
          
          sum_chunk_size_buffer.extend_from_slice(&buf[..n]);
          
          // Check if the end of the stream has been reached
          if n < buf.len() { println!("Buffer not full, EOF reached"); break; }
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          // Stream is not ready yet, try again later
          continue;
        },
        Err(e) => { // Other error occurred
          eprintln!("ERROR: Reading sum chunk size from stream: {}", e);
          *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM.to_string();
          return server_config;
        },
      }
      
      if sum_chunk_size_buffer.ends_with(b"\r\n") {
        // Parse the sum chunk size
        let sum_chunk_size_str = String::from_utf8_lossy(&sum_chunk_size_buffer).trim().to_string();
        let sum_chunk_size = match usize::from_str_radix(&sum_chunk_size_str, 16){
          Ok(v) => v,
          Err(e) =>{
            eprintln!("ERROR: Failed to parse sum_chunk_size_str: {}\n {}", sum_chunk_size_str, e);
            *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE.to_string();
            return server_config;
          }
        };
        
        // Check if the end of the stream has been reached
        if sum_chunk_size == 0
        {
          eprintln!("ERROR: Chunked body with zero sum chunk size");
          *global_error_string = ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE.to_string();
          return server_config;
        }
        break;
        
      }
      
    }
    
    sum_chunk_size_buffer.clear();
    // end of skip the first chunk size line which is sum of all chunks
    
    
    let mut chunk_size = 0;
    let mut chunk_size_buffer = Vec::new();
    
    let mut chunk_buffer = Vec::new();
    
    loop { // read the chunk size
      
      // Check if the timeout has expired
      if start_time.elapsed() >= timeout {
        eprintln!("ERROR: Chunk size read timed out");
        *global_error_string = ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT.to_string();
        return server_config;
      }
      
      // Read from the stream one byte at a time
      match stream.read(&mut buf).await {
        Ok(0) => {
          // EOF reached
          println!("read EOF reached");
          break;
        },
        Ok(n) => {
          
          body_size += n;
          
          // Check if the body size is bigger than client_body_size
          if body_size > client_body_size {
            eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
            *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
            return server_config;
          }
          
          // Successfully read n bytes from stream
          chunk_size_buffer.extend_from_slice(&buf[..n]);
          
          // Check if the end of the stream has been reached
          if n < buf.len() {
            println!("read EOF reached relatively, because buffer not full after read");
            break;
          }
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          // Stream is not ready yet, try again later
          continue;
        },
        Err(e) => {
          // Other error occurred
          eprintln!("ERROR: Reading chunk size from stream: {}", e);
          *global_error_string = ERROR_400_BODY_CHUNK_SIZE_READING_STREAM.to_string();
          return server_config;
        },
      }
      
      // Check if the end of the chunk size has been reached
      if chunk_size_buffer.ends_with(b"\r\n") {
        // Parse the chunk size
        let chunk_size_str = String::from_utf8_lossy(&chunk_size_buffer).trim().to_string();
        chunk_size = match usize::from_str_radix(&chunk_size_str, 16){
          Ok(v) => v,
          Err(e) =>{
            eprintln!("ERROR: Failed to parse chunk_size_str: {}\n {}", chunk_size_str, e);
            *global_error_string = ERROR_400_BODY_CHUNK_SIZE_PARSE.to_string();
            return server_config;
          }
        };
        println!("chunk_size: {}", chunk_size); //todo: remove later
        
        
        // Check if the end of the stream has been reached
        if chunk_size == 0 {
          println!("chunked body read EOF reached");
          break;
        } else { // there is a chunk to read, according to chunk_size
          
          loop { // read the chunk
            
            // Check if the timeout has expired
            if start_time.elapsed() >= timeout {
              println!("ERROR: Chunk body read timed out");
              *global_error_string = ERROR_400_BODY_CHUNK_READ_TIMEOUT.to_string();
              return server_config;
            }
            
            // Read from the stream one byte at a time
            match stream.read(&mut buf).await {
              Ok(0) => {
                // EOF reached
                println!("read EOF reached");
                break;
              },
              Ok(n) => {
                
                body_size += n;
                
                // Check if the body size is bigger than client_body_size
                if body_size > client_body_size {
                  eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
                  *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
                  return server_config;
                }
                
                // Successfully read n bytes from stream
                chunk_buffer.extend_from_slice(&buf[..n]);
                
                // Check if the end of the stream has been reached
                if n < buf.len() {
                  println!("read EOF reached relatively, because buffer not full after read");
                  break;
                }
              },
              Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Stream is not ready yet, try again later
                continue;
              },
              Err(e) => {
                // Other error occurred
                eprintln!("ERROR: Reading chunk from stream: {}", e);
                *global_error_string = ERROR_400_BODY_CHUNK_READING_STREAM.to_string();
                return server_config;
              },
            }
            
            
            // Check if the end of the chunk has been reached
            if chunk_buffer.ends_with(b"\r\n") {
              // Remove the trailing CRLF
              // println!("before truncate chunk_buffer: {:?}", chunk_buffer);
              chunk_buffer.truncate(chunk_buffer.len() - 2);
              // println!("chunk_buffer: {:?}", chunk_buffer);
              
              body_buffer.extend(chunk_buffer.clone());
              
              chunk_buffer.clear();
              chunk_size_buffer.clear();
              chunk_size = 0;
              break;
            }
            else if chunk_buffer.len() > chunk_size + 2 //todo: not obvious, probably
            { // the chunk is broken, because it is bigger than chunk_size
              
              eprintln!("ERROR: Chunk is bigger than chunk_size");
              *global_error_string = ERROR_400_BODY_CHUNK_IS_BIGGER_THAN_CHUNK_SIZE.to_string();
              return server_config;
            }
            
          }
          
        }
        
      }
      
    }
    
  }
  else { // if request is not chunked
    // not clear way to manage not clear standard.
    // To manage cases when there is unchunked body, but
    // without Content-Length header(because this header is not mandatory),
    // try to implement the second timeout for body read in this case.
    // it will be x5 times shorter than the timeout incoming parameter.
    
    println!("THE REQUEST IS NOT CHUNKED");
    
    let mut unchunked_buf:Vec<u8> = Vec::new();
    let dirty_timeout = timeout / 5;
    let dirty_start_time = Instant::now();
    let content_length_header_not_found = !String::from_utf8_lossy(&headers_buffer).contains("Content-Length: ");
    
    let content_length = if content_length_header_not_found {
      println!("ERROR: Content-Length header not found in headers_buffer of unchunked body. Continue with MAX-1 content_length of \ndirty body\n.");
      usize::MAX-1
      
    } else {
      // extract content length from headers_buffer and parse it to usize
      let headers_buffer_slice = headers_buffer.as_slice();
      let headers_str = String::from_utf8_lossy(headers_buffer_slice);
      let content_length_header = 
      headers_str.split("\r\n").find(|&s| s.starts_with("Content-Length: "));
      
      let content_length_str = match content_length_header {
        Some(v) => v.trim_start_matches("Content-Length: "),
        None => {
          eprintln!("ERROR: Failed to get Content-Length header from headers_buffer of unchunked body. Continue with 0 content_length of \ndirty body\n.");
          "0" // help to content_length to be
        } 
      };
      
      match content_length_str.parse() {
        Ok(v) => v,
        Err(e) => {
          eprintln!("ERROR: Failed to parse content_length_str: {}\n {}", content_length_str, e);
          *global_error_string = ERROR_400_HEADERS_FAILED_TO_PARSE.to_string();
          return server_config;
        }
      }
    };
    
    println!("content_length: {}", content_length); //todo: remove later

    loop{
      // check the body_buffer length
      if content_length > 0{
        if body_buffer.len() == content_length{
          break;
        } else if body_buffer.len() > content_length{
          eprintln!("ERROR: body_buffer.len() > content_length");
          *global_error_string = ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH.to_string();
          return server_config;
        }
      }
      
      // Check if the timeout has expired
      if start_time.elapsed() >= timeout {
        eprintln!("ERROR: Body read timed out");
        *global_error_string = ERROR_400_BODY_READ_TIMEOUT.to_string();
        return server_config;
      } else {
        println!("body_buffer.len(): {}", body_buffer.len()); //todo: remove later
        println!("time {} < timeout {}", start_time.elapsed().as_millis(), timeout.as_millis()); //todo: remove later
      }
      
      if content_length_header_not_found // potentilal case of dirty body
      && dirty_start_time.elapsed() >= dirty_timeout
      {
        if body_buffer.len() == 0{
          break; // let is say that there is no body in this case, so continue
        } else {
          eprintln!("ERROR: Dirty body read timed out.\n = File: {}, Line: {}, Column: {}", file!(), line!(), column!()); //todo: remove later
          *global_error_string = ERROR_400_DIRTY_BODY_READ_TIMEOUT.to_string();
          return server_config;
        }
      }
      
      println!(" before \"match stream.read(&mut buf).await {{\"read from the stream one byte at a time"); //todo: remove later FIRES ONCE
      // Read from the stream one byte at a time
      match stream.read_to_end(&mut unchunked_buf).await {
        
        Ok(n) => {
          println!("read one byte NEVER FIRES"); //FIX: remove later. NEVER FIRES
          body_size += n;
          
          // Check if the body size is bigger than client_body_size
          if body_size > client_body_size {
            eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
            *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
            return server_config;
          }
          
          // Successfully read n bytes from stream
          body_buffer.extend_from_slice(&buf[..n]);
          
          // Check if the end of the stream has been reached
          if n < buf.len() {
            println!("read EOF reached relatively, because buffer not full after read");
            break;
          }
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          eprintln!("ERROR: Stream is not ready yet, try again later");
          async_std::task::yield_now().await;
          // Stream is not ready yet, try again later
          continue;
        },
        Err(e) => {
          // Other error occurred
          eprintln!("ERROR: Reading from stream: {}", e);
          *global_error_string = ERROR_400_BODY_READING_STREAM.to_string();
          return server_config;
        },
      }

      println!(" AFTER \"match stream.read(&mut buf).await {{\"read from the stream one byte at a time"); //fix: remove later NEVER FIRES
      
    }
    
  }

*/
  server_config
  
}

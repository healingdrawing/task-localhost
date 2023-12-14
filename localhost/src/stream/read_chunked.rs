use std::error::Error;
use std::time::{Instant, Duration};

use async_std::net::TcpStream;



pub async fn read_chunked(stream: &mut TcpStream, body_buffer: &mut Vec<u8>, timeout: Duration) {
  
  /*
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
  */
  
  
  todo!("read_chunked")
  
}
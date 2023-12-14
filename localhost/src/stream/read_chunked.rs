use std::error::Error;
use std::fmt::format;
use std::time::{Instant, Duration};

use async_std::io;
use async_std::net::TcpStream;
use futures::AsyncReadExt;

use crate::debug::{append_to_file, DEBUG};
use crate::stream::errors::{ERROR_400_HEADERS_READ_TIMEOUT, ERROR_400_HEADERS_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE, ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNK_READ_TIMEOUT, ERROR_400_BODY_CHUNK_READING_STREAM, ERROR_400_BODY_CHUNK_IS_BIGGER_THAN_CHUNK_SIZE, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH, ERROR_400_BODY_READ_TIMEOUT, ERROR_400_DIRTY_BODY_READ_TIMEOUT, ERROR_400_BODY_READING_STREAM, ERROR_413_BODY_SIZE_LIMIT};



pub async fn read_chunked(
  stream: &mut TcpStream,
  body_buffer: &mut Vec<u8>,
  client_body_size: usize,
  has_content_length_header: bool,
  content_length: usize,
  timeout: Duration,
  global_error_string: &mut String,
) {
  
  
  println!("THE REQUEST IS CHUNKED");
  
  let start_time = Instant::now();
  
  let mut body_size = 0;
  
  let mut sum_chunk_size_buffer = Vec::new();
  
  let mut sum_chunk_size = 0;
  
  // since timeout implemented, skip the first chunk size line which is sum of all chunks
  loop { // read the sum chunk size
    
    // async time sleep for 2 ms, for some append_to_file() prints cases binded to time.
    // with big body can easily break the reading timeouts. So, use carefully.
    if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
    
    // Check if the timeout has expired
    if start_time.elapsed() >= timeout {
      eprintln!("ERROR: Sum chunk size read timed out");
      *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT.to_string();
      return
    }
    
    let mut buf = [0; 1];
    
    // Read from the stream one byte at a time
    match stream.read(&mut buf).await {
      Ok(0) => {
        append_to_file(&format!("read EOF reached. Read sum chunk size")).await;
        break;
      },
      Ok(n) => { // Successfully read n bytes from stream
        
        body_size += n;
        
        // Check if the body size is bigger than client_body_size
        if body_size > client_body_size {
          eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
          *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
          return
        }
        
        sum_chunk_size_buffer.extend_from_slice(&buf[..n]);
        
        // Check if the end of the stream has been reached
        if n < buf.len() {
          append_to_file(&format!("read EOF reached relatively.\nBuffer not full after read. Read sum chunk size")).await;
          return //todo: not obvious, probably
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Stream is not ready yet, try again later
        continue;
      },
      Err(e) => { // Other error occurred
        eprintln!("ERROR: Reading sum chunk size from stream: {}", e);
        *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM.to_string();
        return
      },
    }
    
    if sum_chunk_size_buffer.ends_with(b"\r\n") {
      
      // Parse the sum chunk size
      let sum_chunk_size_str = String::from_utf8_lossy(&sum_chunk_size_buffer).trim().to_string();
      
      sum_chunk_size = match usize::from_str_radix(&sum_chunk_size_str, 16){
        Ok(v) => v,
        Err(e) =>{
          eprintln!("ERROR: Failed to parse sum_chunk_size_str: {}\n {}", sum_chunk_size_str, e);
          *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE.to_string();
          return
        }
      };
      
      
      break;
      
    }
    
  }
  
  if sum_chunk_size == 0 {
    eprintln!("ERROR: Chunked body with zero sum chunk size");
    *global_error_string = ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE.to_string();
    return
  }
  
  sum_chunk_size_buffer.clear(); // now more memory is in safe, rust creators can sleep well
  // ------------------------------------
  // End of skip the first chunk size line which is sum of all chunks.
  // Description in the beginning of the loop above
  // ------------------------------------
  
  let mut chunk_size = 0;
  let mut chunk_size_buffer = Vec::new();
  
  let mut chunk_buffer = Vec::new();
  
  loop { // read the chunk size
    
    // async time sleep for 2 ms, for some append_to_file() prints cases binded to time.
    // with big body can easily break the reading timeouts. So, use carefully.
    if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
    
    // Check if the timeout has expired
    if start_time.elapsed() >= timeout {
      eprintln!("ERROR: Chunk size read timed out");
      *global_error_string = ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT.to_string();
      return
    }
    
    let mut buf = [0; 1];
    
    // Read from the stream one byte at a time
    match stream.read(&mut buf).await {
      Ok(0) => {
        // EOF reached
        append_to_file(&format!("read EOF reached. Read chunk size")).await;
        break;
      },
      Ok(n) => {
        
        body_size += n;
        
        // Check if the body size is bigger than client_body_size
        if body_size > client_body_size {
          eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
          *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
          return
        }
        
        // Successfully read n bytes from stream
        chunk_size_buffer.extend_from_slice(&buf[..n]);
        
        // Check if the end of the stream has been reached
        if n < buf.len() {
          append_to_file(&format!("read EOF reached relatively.\nBuffer not full after read. Read chunk size")).await;
          return //todo: not obvious, probably
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
        return
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
          return
        }
      };
      append_to_file(&format!("chunk_size: {}", chunk_size)).await;
      
      
      // Check if the end of the stream has been reached
      if chunk_size == 0 {
        append_to_file(&format!("chunked body read EOF reached")).await;
        return
      } else { // there is a chunk to read, according to chunk_size
        
        loop { // read the chunk
          
          // async time sleep for 2 ms, for some append_to_file() prints cases binded to time.
          // with big body can easily break the reading timeouts. So, use carefully.
          if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
          
          // Check if the timeout has expired
          if start_time.elapsed() >= timeout {
            println!("ERROR: Chunk body read timed out");
            *global_error_string = ERROR_400_BODY_CHUNK_READ_TIMEOUT.to_string();
            return
          }
          
          let mut buf = [0; 1];
          
          // Read from the stream one byte at a time
          match stream.read(&mut buf).await {
            Ok(0) => {
              append_to_file(&format!("read EOF reached")).await;
              break;
            },
            Ok(n) => {
              
              body_size += n;
              
              // Check if the body size is bigger than client_body_size
              if body_size > client_body_size {
                eprintln!("ERROR: Body size is bigger than client_body_size limit: {} > {}", body_size, client_body_size);
                *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
                return
              }
              
              // Successfully read n bytes from stream
              chunk_buffer.extend_from_slice(&buf[..n]);
              
              // Check if the end of the stream has been reached
              if n < buf.len() {
                append_to_file(&format!("read EOF reached relatively, Buffer not full after read. Read chunk")).await;
                return //todo: not obvious, probably
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
              return
            },
          }
          
          
          // Check if the end of the chunk has been reached
          if chunk_buffer.ends_with(b"\r\n") {
            // Remove the trailing CRLF
            append_to_file(&format!("before truncate chunk_buffer: {:?}", chunk_buffer)).await;
            chunk_buffer.truncate(chunk_buffer.len() - 2);
            append_to_file(&format!("chunk_buffer: {:?}", chunk_buffer)).await;
            
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
            return
          }
          
        }
        
      }
      
    }
    
  }
  
}
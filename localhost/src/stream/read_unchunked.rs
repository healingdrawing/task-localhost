use std::error::Error;
use std::time::{Instant, Duration};

use async_std::io;
use async_std::net::TcpStream;
use futures::AsyncReadExt;

use crate::debug::append_to_file;
use crate::stream::errors::{ERROR_400_HEADERS_READ_TIMEOUT, ERROR_400_HEADERS_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE, ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNK_READ_TIMEOUT, ERROR_400_BODY_CHUNK_READING_STREAM, ERROR_400_BODY_CHUNK_IS_BIGGER_THAN_CHUNK_SIZE, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH, ERROR_400_BODY_READ_TIMEOUT, ERROR_400_DIRTY_BODY_READ_TIMEOUT, ERROR_400_BODY_READING_STREAM, ERROR_413_BODY_SIZE_LIMIT};


pub async fn read_unchunked(
  stream: &mut TcpStream,
  headers_buffer: &mut Vec<u8>,
  body_buffer: &mut Vec<u8>,
  client_body_size: usize,
  has_content_length_header: bool,
  content_length: usize,
  timeout: Duration,
  global_error_string: &mut String,
) {
  
  // if request is not chunked
  // not clear way to manage not clear standard.
  // To manage cases when there is unchunked body, but
  // without Content-Length header(because this header is not mandatory),
  // try to implement the second timeout for body read in this case.
  // it will be x5 times shorter than the timeout incoming parameter.
  
  println!("THE REQUEST IS NOT CHUNKED");
  
  println!("\nstream: {:?}\ninside read body", stream); //todo: remove later
  
  // Start the timer for body read
  let start_time = Instant::now();
  let mut body_size = 0;
  
  let dirty_timeout = timeout / 5;
  let dirty_start_time = Instant::now();
  
  
  
  // check content_length not more than client_body_size
  if content_length > client_body_size {
    eprintln!("ERROR: Content-Length header value is bigger than client_body_size limit: {} > {}", content_length, client_body_size);
    *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
    return
  }
  
  if has_content_length_header && content_length < 1 {
    eprintln!("ERROR: There is no positive content_length value. Continue without reading body.");
    return
  }
  
  loop{
    // async time sleep for 2 ms
    async_std::task::sleep(Duration::from_millis(2)).await; //todo: remove later ?
    
    // check the body_buffer length
    
    if body_buffer.len() == content_length{
      return
    } else if body_buffer.len() > content_length{
      eprintln!("ERROR: body_buffer.len() > content_length");
      *global_error_string = ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH.to_string();
      return 
    }
    
    // Check if the timeout has expired
    if start_time.elapsed() >= timeout {
      eprintln!("ERROR: Body read timed out");
      append_to_file("ERROR: Body read timed out").await;
      *global_error_string = ERROR_400_BODY_READ_TIMEOUT.to_string();
      return 
    } else {
      append_to_file(&format!("body_buffer.len(): {}", body_buffer.len())).await; //todo: remove later
      append_to_file(&format!("time {} < timeout {}", start_time.elapsed().as_millis(), timeout.as_millis())).await; //todo: remove later
    }
    
    if !has_content_length_header // potentilal case of dirty body
    && dirty_start_time.elapsed() >= dirty_timeout
    {
      if body_buffer.len() < 1{
        return // let is say that there is no body in this case, so continue
      } else {
        eprintln!("ERROR: Dirty body read timed out.\n = File: {}, Line: {}, Column: {}", file!(), line!(), column!()); //todo: remove later
        *global_error_string = ERROR_400_DIRTY_BODY_READ_TIMEOUT.to_string();
        return 
      }
    }
    
    let mut buf = [0; 1];
    
    // Read from the stream one byte at a time
    match stream.read(&mut buf).await {
      Ok(0) => {
        // EOF reached
        println!("read EOF reached");
        return
      },
      Ok(n) => {
        // Successfully read n bytes from stream
        // println!("attempt to read {} bytes from stream", n);
        body_buffer.extend_from_slice(&buf[..n]);
        // println!("after read body buffer size: {}", body_buffer.len());
        // println!("after read body buffer: {:?}", body_buffer);
        // println!("after read body buffer to string: {:?}", String::from_utf8(body_buffer.clone()));
        // Check if the end of the stream has been reached
        if n < buf.len() {
          println!("read EOF reached relatively, because buffer not full after read");
          return
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Stream is not ready yet, try again later
        continue;
      },
      Err(e) => {
        // Other error occurred
        eprintln!("ERROR: Reading body from stream: {}", e);
        *global_error_string = ERROR_400_BODY_READING_STREAM.to_string();
        return
      },
    }
    
  }
  
}

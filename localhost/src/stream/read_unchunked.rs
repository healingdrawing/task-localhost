use std::time::{Instant, Duration};

use async_std::io;
use async_std::net::TcpStream;
use futures::AsyncReadExt;

use crate::debug::{append_to_file, DEBUG};
use crate::stream::errors::{ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH, ERROR_400_BODY_READ_TIMEOUT, ERROR_400_BODY_READING_STREAM, ERROR_413_BODY_SIZE_LIMIT};


pub async fn read_unchunked(
  stream: &mut TcpStream,
  body_buffer: &mut Vec<u8>,
  client_body_size: usize,
  content_length: usize,
  timeout: Duration,
  global_error_string: &mut String,
) {
  
  append_to_file(
    "=======================\n= NOT CHUNKED REQUEST =\n======================="
  ).await;
  
  append_to_file(&format!("\nstream: {:?}\ninside read body", stream)).await;
  
  // Start the timer for body read
  let start_time = Instant::now();
  
  if content_length > client_body_size {
    eprintln!("ERROR: Content-Length header value is greater than client_body_size limit: {} > {}", content_length, client_body_size);
    *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
    return
  }
  
  loop{
    // async time sleep for 2 ms, for some append_to_file() prints cases binded to time.
    // with big body can easily break the reading timeouts. So, use carefully.
    if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }

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
      append_to_file(&format!("body_buffer.len(): {}", body_buffer.len())).await;
      append_to_file(&format!("time {} < timeout {}", start_time.elapsed().as_millis(), timeout.as_millis())).await;
    }
    
    let mut buf = [0; 1];
    
    // Read from the stream one byte at a time
    match stream.read(&mut buf).await {
      Ok(0) => {
        append_to_file("read EOF reached. Read unchunked body size").await;
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
          append_to_file("read EOF reached relatively, because buffer not full after read").await;
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

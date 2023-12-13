use std::error::Error;
use std::time::{Instant, Duration};

use async_std::net::TcpStream;



pub async fn read_chunked(stream: &mut TcpStream, body_buffer: &mut Vec<u8>, timeout: Duration) {
  // Start the timer for body read
  let start_time = Instant::now();

  todo!("read_chunked")

}
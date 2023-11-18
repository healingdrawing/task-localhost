use std::time::{Instant, Duration};
use std::io::{self, Read};
use mio::net::TcpStream;

/// Read from the stream until timeout or EOF
/// 
/// returns a tuple of two vectors: (headers_buffer, body_buffer)
pub fn read_with_timeout(stream: &mut TcpStream, timeout: Duration) -> io::Result<(Vec<u8>,Vec<u8>)> {
  todo!("INSIDE read_with_timeout. Implement double vec in result(headers,body), and so on");
  println!("INSIDE read_with_timeout");
  // Start the timer
  let start_time = Instant::now();
  println!("start_time: {:?}", start_time);
  
  // Read from the stream until timeout or EOF
  let mut buf = [0; 1];
  
  // collect request headers section
  let mut headers_buffer = Vec::new();
  
  loop {
    // Check if the timeout has expired
    if start_time.elapsed() >= timeout {
      println!("headers read timed out");
      return Err(io::Error::new(io::ErrorKind::TimedOut, "headers read timed out"));
    }
    
    match stream.read(&mut buf) {
      Ok(0) => {
        // EOF reached
        println!("read EOF reached");
        break;
      },
      Ok(n) => {
        // Successfully read n bytes from stream
        println!("attempt to read {} bytes from stream", n);
        headers_buffer.extend_from_slice(&buf[..n]);
        println!("after read headers buffer size: {}", headers_buffer.len());
        println!("after read headers buffer: {:?}", headers_buffer);
        println!("after read headers buffer to string: {:?}", String::from_utf8(headers_buffer.clone()));
        // Check if the end of the stream has been reached
        if n < buf.len() {
          println!("read EOF reached relatively, because buffer not full after read");
          break;
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Stream is not ready yet, try again later
        // println!("= BANG! this crap happens...read would block");
        continue;
      },
      Err(e) => {
        // Other error occurred
        eprintln!("Error reading headers from stream: {}", e);
        return Err(e);
      },
    }
    
    
    if headers_buffer.ends_with(b"\r\n\r\n") {
      break;
    }
  }
  
  let is_chunked = String::from_utf8_lossy(&headers_buffer).contains("Transfer-Encoding: chunked");
  
  // collect request body section
  let mut body_buffer = Vec::new();
  
  if is_chunked {
    println!("THE REQUEST IS CHUNKED: {}", is_chunked); //todo: remove later
    // let mut full = String::new();
    // stream.read_to_string(&mut full);
    // println!("stream :\n{:?}", full.split("").collect::<Vec<_>>());
    
    
    let mut sum_chunk_size_buffer = Vec::new();
    
    loop { // skip the first chunk size line which is sum of all chunks
      // Check if the timeout has expired
      if start_time.elapsed() >= timeout {
        println!("sum chunk size body read timed out");
        return Err(io::Error::new(io::ErrorKind::TimedOut, "sum chunk size body read timed out"));
      }
      
      // Read from the stream one byte at a time
      match stream.read(&mut buf) {
        Ok(0) => { println!("read EOF reached"); break },
        Ok(n) => { // Successfully read n bytes from stream
          
          println!("attempt to read {} bytes from stream", n);
          sum_chunk_size_buffer.extend_from_slice(&buf[..n]);
          println!("after read sum chunk size buffer size: {}", sum_chunk_size_buffer.len());
          println!("after read sum chunk size buffer: {:?}", sum_chunk_size_buffer);
          println!("after read sum chunk size buffer to string: {:?}", String::from_utf8(sum_chunk_size_buffer.clone()));
          
          // Check if the end of the stream has been reached
          if n < buf.len() { println!("buffer not full, EOF reached"); break; }
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          // Stream is not ready yet, try again later
          // println!("= BANG! this crap happens...read would block");
          continue;
        },
        Err(e) => { // Other error occurred
          eprintln!("Error reading chunk size from stream: {}", e);
          return Err(e);
        },
      }
      
      if sum_chunk_size_buffer.ends_with(b"\r\n") {
        // Parse the sum chunk size
        println!("sum_chunk_size_buffer: {:?}", sum_chunk_size_buffer); //todo: remove later
        let sum_chunk_size_str = String::from_utf8_lossy(&sum_chunk_size_buffer).trim().to_string();
        println!("sum_chunk_size_str: {}", sum_chunk_size_str);
        let sum_chunk_size = usize::from_str_radix(&sum_chunk_size_str, 16).unwrap();
        println!("sum chunk_size: {}", sum_chunk_size);
        
        // Check if the end of the stream has been reached
        if sum_chunk_size == 0
        {
          return Err(io::Error::new(io::ErrorKind::InvalidData, "sum chunk size is zero"));//todo: check this
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
        println!("chunk size body read timed out");
        return Err(io::Error::new(io::ErrorKind::TimedOut, "chunk size body read timed out"));
      }
      
      // Read from the stream one byte at a time
      match stream.read(&mut buf) {
        Ok(0) => {
          // EOF reached
          println!("read EOF reached");
          break;
        },
        Ok(n) => {
          // Successfully read n bytes from stream
          println!("attempt to read {} bytes from stream", n);
          chunk_size_buffer.extend_from_slice(&buf[..n]);
          println!("after read chunk size buffer size: {}", chunk_size_buffer.len());
          println!("after read chunk size buffer: {:?}", chunk_size_buffer);
          println!("after read chunk size buffer to string: {:?}", String::from_utf8(chunk_size_buffer.clone()));
          // Check if the end of the stream has been reached
          if n < buf.len() {
            println!("read EOF reached relatively, because buffer not full after read");
            break;
          }
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          // Stream is not ready yet, try again later
          // println!("= BANG! this crap happens...read would block");
          continue;
        },
        Err(e) => {
          // Other error occurred
          eprintln!("Error reading chunk size from stream: {}", e);
          return Err(e);
        },
      }
      
      
      // Check if the end of the chunk size has been reached
      if chunk_size_buffer.ends_with(b"\r\n") {
        // Parse the chunk size
        println!("chunk_size_buffer: {:?}", chunk_size_buffer);
        let chunk_size_str = String::from_utf8_lossy(&chunk_size_buffer).trim().to_string();
        chunk_size = usize::from_str_radix(&chunk_size_str, 16).unwrap();
        println!("chunk_size: {}", chunk_size);
        
        
        // Check if the end of the stream has been reached
        if chunk_size == 0 {
          println!("chunked body read EOF reached");
          break;
        } else { // there is a chunk to read, according to chunk_size
          
          loop { // read the chunk
            
            // Check if the timeout has expired
            if start_time.elapsed() >= timeout {
              println!("chunk body read timed out");
              return Err(io::Error::new(io::ErrorKind::TimedOut, "chunk body read timed out"));
            }
            
            // Read from the stream one byte at a time
            match stream.read(&mut buf) {
              Ok(0) => {
                // EOF reached
                println!("read EOF reached");
                break;
              },
              Ok(n) => {
                // Successfully read n bytes from stream
                println!("attempt to read {} bytes from stream", n);
                chunk_buffer.extend_from_slice(&buf[..n]);
                println!("after read chunk buffer size: {}", chunk_buffer.len());
                println!("after read chunk buffer: {:?}", chunk_buffer);
                println!("after read chunk buffer to string: {:?}", String::from_utf8(chunk_buffer.clone()));
                // Check if the end of the stream has been reached
                if n < buf.len() {
                  println!("read EOF reached relatively, because buffer not full after read");
                  break;
                }
              },
              Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Stream is not ready yet, try again later
                // println!("= BANG! this crap happens...read would block");
                continue;
              },
              Err(e) => {
                // Other error occurred
                eprintln!("Error reading chunk from stream: {}", e);
                return Err(e);
              },
            }
            
            
            // Check if the end of the chunk has been reached
            if chunk_buffer.ends_with(b"\r\n") {
              // Remove the trailing CRLF
              println!("before truncate chunk_buffer: {:?}", chunk_buffer);
              chunk_buffer.truncate(chunk_buffer.len() - 2);
              println!("chunk_buffer: {:?}", chunk_buffer);
              println!("chunk_buffer to string: {:?}", String::from_utf8(chunk_buffer.clone()));
              body_buffer.extend(chunk_buffer.clone());
              
              chunk_buffer.clear();
              chunk_size_buffer.clear();
              chunk_size = 0;
              break;
            }
            else if chunk_buffer.len() > chunk_size + 2 //todo: check this
            { // the chunk is broken, because it is bigger than chunk_size
              println!("{} > {}", chunk_buffer.len(), chunk_size + 2);
              println!("= FAIL =");
              println!("chunk_buffer: {:?}", chunk_buffer);
              println!("chunk_buffer to string: {:?}", String::from_utf8(chunk_buffer.clone()));
              println!("chunk_buffer len: {}", chunk_buffer.len());
              println!("chunk_size: {}", chunk_size);
              return Err(io::Error::new(io::ErrorKind::InvalidData, "chunk is bigger than chunk_size"));
            }
            
          }
          
        }
        
      }
      
    }
    
  }
  else { // if request is not chunked
    loop{
      // Check if the timeout has expired
      if start_time.elapsed() >= timeout {
        println!("body read timed out");
        return Err(io::Error::new(io::ErrorKind::TimedOut, "body read timed out"));
      }
      
      // Read from the stream one byte at a time
      match stream.read(&mut buf) {
        Ok(0) => {
          // EOF reached
          println!("read EOF reached");
          break;
        },
        Ok(n) => {
          // Successfully read n bytes from stream
          println!("attempt to read {} bytes from stream", n);
          body_buffer.extend_from_slice(&buf[..n]);
          println!("after read buffer size: {}", body_buffer.len());
          println!("after read buffer: {:?}", body_buffer);
          println!("after read buffer to string: {:?}", String::from_utf8(body_buffer.clone()));
          // Check if the end of the stream has been reached
          if n < buf.len() {
            println!("read EOF reached relatively, because buffer not full after read");
            break;
          }
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
          // Stream is not ready yet, try again later
          // println!("= BANG! this crap happens...read would block");
          continue;
        },
        Err(e) => {
          // Other error occurred
          eprintln!("Error reading from stream: {}", e);
          return Err(e);
        },
      }
      
    }
  }
  
  
  println!("read {} bytes from stream", body_buffer.len());
  println!("Raw incoming buffer to string: {:?}", String::from_utf8(body_buffer.clone()));
  
  Ok((headers_buffer, body_buffer))
}


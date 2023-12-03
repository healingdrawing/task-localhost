use std::{fs::{File, OpenOptions}, io::{self, Write}};

pub const DEBUG: bool = false; //set to false to disable debug.txt stuff

pub const DEBUG_FILE: &str = "debug.txt";

pub fn try_recreate_file_according_to_value_of_debug_boolean() -> io::Result<()> {
  if DEBUG { File::create(DEBUG_FILE)?; }
  Ok(())
}

// Function to append data to a file
pub fn append_to_file(data: &str) -> io::Result<()> {
  if DEBUG {
    let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(DEBUG_FILE)?;
    
    let mut writer = io::BufWriter::new(file);
    writeln!(writer, "{}", data)?;
  }
  Ok(())

}

/// create a file with name "something" inside uploads folder,
/// 
/// to allow the user remove this file using DELETE method
/// 
/// as audit question requires
pub fn create_something_in_uploads_folder() -> io::Result<()> {
  File::create("uploads/something")?; 
  Ok(())
}
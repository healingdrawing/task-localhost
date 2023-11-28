use std::path::Path;

//ok it works
pub fn dummy_check_file_path(){
  let path = Path::new("debug.txt");
   if path.exists() {
       println!("File exists! {:?}", path);
   } else {
       println!("File does not exist! {:?}", path);
   }
}

//todo: implement the function to check the file by relative path in time of server configuration check, before run the server. Should be used to prevent run without required files described in config etc. Later after server run, if some files will be removed, server , according to task requirements must continue to work. To implement this, the emergency server error 500 page must be hardcoded into executable, and the server must be able to serve it without any files.
use std::{path::PathBuf, fs};

pub fn generate_uploads_html(absolute_path: &PathBuf) -> String {
  let mut html = String::new();
  html.push_str("<h1>Uploads</h1>");
  html.push_str("<ul>");
  for entry in fs::read_dir(absolute_path).unwrap() {
      let entry = entry.unwrap();
      let path = entry.path();
      if path.is_file() {
          let file_name = path.file_name().unwrap().to_str().unwrap();
          html.push_str(&format!("<li><a href=\"/uploads/{}\">{}</a>", file_name, file_name));
          html.push_str("<form method=\"DELETE\" action=\"/uploads\">");
          html.push_str(&format!("<input type=\"hidden\" name=\"file\" value=\"{}\">", file_name));
          html.push_str("<input type=\"submit\" value=\"Delete\">");
          html.push_str("</form>");
          html.push_str("</li>");
      }
  }
  html.push_str("</ul>");
  html.push_str("<form method=\"POST\" action=\"/uploads\">");
  html.push_str("<input type=\"file\" name=\"file\">");
  html.push_str("<input type=\"submit\" value=\"Upload\">");
  html.push_str("</form>");
  html
 }


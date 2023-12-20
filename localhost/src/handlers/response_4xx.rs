use async_std::path::PathBuf;

use http::{Request, Response, StatusCode};

use crate::server::core::ServerConfig;
use crate::handlers::response_500::custom_response_500;

const ALLOWED_4XX_STATUS_CODES: [StatusCode; 5] = [
  StatusCode::BAD_REQUEST,
  StatusCode::FORBIDDEN,
  StatusCode::NOT_FOUND, //managed inside handle_all
  StatusCode::METHOD_NOT_ALLOWED, //managed inside handle_all
  StatusCode::PAYLOAD_TOO_LARGE,
];
/// return custom 4xx error response.
/// According to task, the next custom error 4xx are required to handle:
/// 400,403,404,405,413
/// if error happens, then return custom_response_500
pub async fn custom_response_4xx(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
  status_code: StatusCode,
) -> Response<Vec<u8>>{

  // check status code is in 4xx list 400,403,404,405,413
  if !ALLOWED_4XX_STATUS_CODES.contains(&status_code){
    eprintln!("ERROR: Internal Server Error\ncustom_response_4xx: status code {:?}\nis not in 4xx list {:?}", status_code, ALLOWED_4XX_STATUS_CODES);
    return custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    ).await
  }

  let error_page_path = zero_path_buf
  .join("static")
  .join(server_config.error_pages_prefix.clone())
  .join(status_code.as_str().to_string() + ".html");
  
  // read the error page. if error, then return custom_response_500
  let error_page_content = match std::fs::read(error_page_path){
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to read error page: {}", e);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      ).await
    }
  };

  let response = match Response::builder()
  .status(status_code)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(error_page_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to create custom 4xx response: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config,
      ).await
    }
  };
  
  response

}
pub enum HttpResponse {
    // 200 OK with a body
    Ok { content_type: String, body: String },

    // 201 Created
    Created,

    // 404 Not Found
    NotFound,

    // 400 Bad Request with a reason
    BadRequest(String),
}

impl HttpResponse {
    pub fn to_http_string(&self) -> String {
        match self {
            HttpResponse::Ok{content_type, body} => http_response(200, "OK", content_type, body),
            HttpResponse::Created => http_response(201, "OK", "text/plain", ""),
            HttpResponse::NotFound => http_response(404, "Not Found", "", ""),
            HttpResponse::BadRequest(reason) => http_response(400, "Bad Request", "", reason),
        }
    }
}

fn http_response(status_code: u16, status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status,
        content_type,
        body.len(),
        body
    )
}

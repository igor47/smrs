use std::{env, io};
use std::io::prelude::*;

mod token;
use token::{TokenType, generate};

const SESSION_COOKIE_NAME: &str = "smrs_session_id";
const SESSION_COOKIE_MAX_AGE: u64 = 60 * 60 * 24 * 365; // 1 year

#[derive(Debug)]
struct Header {
    key: String,
    value: String,
}

#[derive(Debug)]
struct Body {
    content_type: String,
    content_length: u64,
    data: String,
}

#[derive(Debug)]
struct Request {
    method: String,
    uri: String,
    headers: Vec<Header>,
    body: Option<Body>,
}

impl Request {
    fn from_env() -> Result<Request, &'static str> {
        let method = match env::var("REQUEST_METHOD") {
            Ok(method) => method,
            Err(_) => return Err("REQUEST_METHOD not set"),
        };

        let uri = match env::var("REQUEST_URI") {
            Ok(uri) => uri,
            Err(_) => return Err("REQUEST_URI not set"),
        };

        Ok(Request {
            method,
            uri,
            headers: {
                let mut headers = Vec::new();
                for (key, value) in env::vars() {
                    if key.starts_with("HTTP_") {
                        let key = &key[5..];
                        let key = key.replace("_", "-");
                        let key = key.to_lowercase();
                        headers.push(Header { key, value });
                    }
                };

                headers
            },
            body: {
                match env::var("CONTENT_LENGTH") {
                    Ok(content_length) => {
                        let content_length = match content_length.parse::<u64>() {
                            Ok(content_length) => content_length,
                            Err(_) => return Err("Invalid CONTENT_LENGTH"),
                        };

                        let content_type = match env::var("CONTENT_TYPE") {
                            Ok(content_type) => content_type,
                            Err(_) => return Err("CONTENT_TYPE not set"),
                        };

                        let mut data = String::new();
                        if let Err(_) = io::stdin().read_to_string(&mut data) {
                            return Err("Failed to read request body");
                        }

                        Some(Body { content_type, content_length, data })
                    },
                    Err(_) => None,
                }
            }
        })
    }

    fn cookie(&self, name: &str) -> Option<&str> {
        for header in &self.headers {
            if header.key == "cookie" {
                let cookies = header.value.split("; ");
                for cookie in cookies {
                    let mut parts = cookie.split("=");
                    let cookie_name = parts.next().unwrap();
                    let cookie_value = parts.next().unwrap();
                    if cookie_name == name {
                        return Some(cookie_value);
                    }
                }
            }
        }

        None
    }
}

struct Response {
    status: u16,
    headers: Vec<Header>,
    body: String,

    session_set: bool,
}

impl Response {
    fn new() -> Response {
        let mut response = Response {
            status: 200,
            headers: Vec::new(),
            body: String::new(),
            session_set: false,
        };
        response.add_header("SMRS-Version", "0.0.1");
        response
    }

    fn add_header(&mut self, key: &str, value: &str) {
        self.headers.push(Header { key: key.to_string(), value: value.to_string() });
    }

    fn set_session(&mut self, session_id: Option<&str>) {
        let generated: String;

        let session_id = match session_id {
            Some(session_id) => session_id,
            None => {
                generated = generate(TokenType::Session);
                &generated
            }
        };
        self.add_header(
            "Set-Cookie",
            &format!(
                "{}={}; SameSite=strict; Secure; Max-Age={}",
                SESSION_COOKIE_NAME,
                session_id,
                SESSION_COOKIE_MAX_AGE
            )
        );
        self.session_set = true;
    }

    fn default_session(&mut self, request: &Request) {
        if self.session_set {
            return;
        }
        self.set_session(request.cookie(SESSION_COOKIE_NAME));
    }

    fn send(&self) {
        println!("Status: {0}", self.status);
        for header in &self.headers {
            println!("{0}: {1}", header.key, header.value);
        }
        println!("\n{0}", self.body);
    }
}

fn print_env(request: &Request, response: &mut Response) {
    response.add_header("Content-Type", "text/plain");

    for (key, value) in env::vars() {
        response.body.push_str(&format!("{0}: {1}\n", key, value));
    }

    response.body.push_str(&format!("\nRequest info: {:#?}", request));
}

fn main() {
    let mut response = Response::new();
    let request = Request::from_env();
    let request = match request {
        Ok(request) => request,
        Err(err) => {
            response.status = 400;
            response.body = String::from(format!("Invalid request: {}", err));
            response.send();
            return;
        }
    };

    print_env(&request, &mut response);

    response.default_session(&request);
    response.send();
}

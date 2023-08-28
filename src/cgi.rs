use std::{env, io};
use std::io::prelude::*;

use crate::token::{TokenType, generate};

const SESSION_COOKIE_NAME: &str = "smrs_session_id";
const SESSION_COOKIE_MAX_AGE: u64 = 60 * 60 * 24 * 365; // 1 year

#[derive(Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Body {
    pub content_type: String,
    pub content_length: u64,
    pub data: String,
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub uri: String,
    pub headers: Vec<Header>,
    pub body: Option<Body>,
}

impl Request {
    pub fn from_env() -> Result<Request, &'static str> {
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

    pub fn cookie(&self, name: &str) -> Option<&str> {
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

    pub fn path(&self) -> &str {
        let mut parts = self.uri.split("?");
        parts.next().unwrap()
    }
}

pub struct Response {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: String,

    pub session: Option<String>,
}

impl Response {
    pub fn new() -> Response {
        let mut response = Response {
            status: 200,
            headers: Vec::new(),
            body: String::new(),
            session: None,
        };
        response.add_header("SMRS-Version", "0.0.1");
        response
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.push(Header { key: key.to_string(), value: value.to_string() });
    }

    pub fn set_session(&mut self, session_id: Option<&str>) {
        if let Some(_) = self.session {
            return;
        }

        match session_id {
            Some(session_id) => self.session = Some(session_id.to_string()),
            None => self.session = Some(generate(TokenType::Session)),
        };

        self.add_header(
            "Set-Cookie",
            &format!(
                "{}={}; SameSite=strict; Secure; Max-Age={}",
                SESSION_COOKIE_NAME,
                self.session.as_ref().unwrap(),
                SESSION_COOKIE_MAX_AGE
            )
        );
    }

    pub fn default_session(&mut self, request: &Request) {
        if let Some(_) = self.session {
            return;
        }
        self.set_session(request.cookie(SESSION_COOKIE_NAME));
    }

    pub fn send(&self) {
        println!("Status: {0}", self.status);
        for header in &self.headers {
            println!("{0}: {1}", header.key, header.value);
        }
        println!("\n{0}", self.body);
    }
}


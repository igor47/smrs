mod token;
mod cgi;
mod store;

use std::env;
use serde::{Deserialize, Serialize};

use cgi::{Request, Response};
use store::{open};

fn print_env(request: &Request, response: &mut Response) {
    response.add_header("Content-Type", "text/plain");

    for (key, value) in env::vars() {
        response.body.push_str(&format!("{0}: {1}\n", key, value));
    }

    response.body.push_str(&format!("\nRequest info: {:#?}", request));
}

fn not_found(response: &mut Response) {
    response.status = 404;
    response.add_header("Content-Type", "text/plain");
    response.body = String::from("Not found");
}

#[derive(Serialize)]
struct ShowSessionResp {
    session: String,
}

fn show_session(request: &Request, response: &mut Response) {
    response.default_session(&request);
    response.json(&ShowSessionResp {
        session: response.session.as_ref().unwrap().to_string(),
    });
}

#[derive(Deserialize)]
struct SetSessionReq {
    session: String,
}

fn set_session(request: &Request, response: &mut Response) {
    let set_session = request.json::<SetSessionReq>();
    match set_session {
        Ok(set_session) => {
            response.set_session(Some(&set_session.session));
            response.json(&ShowSessionResp {
                session: response.session.as_ref().unwrap().to_string(),
            });
        },
        Err(err) => {
            response.status = 400;
            response.add_header("Content-Type", "text/plain");
            response.body = String::from(format!("Invalid request: {}", err));
        }
    }
}

fn shorten(request: &Request, response: &mut Response) {
    let db = match open() {
        Ok(db) => db,
        Err(err) => {
            response.status = 500;
            response.body = String::from(format!("Failed to open database: {}", err));
            return;
        }
    };
}

fn redirect(request: &Request, response: &mut Response) {
}

fn list(request: &Request, response: &mut Response) {
}

fn forget(request: &Request, response: &mut Response) {
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

    // process our routes (this is a very simple router)
    if (request.method == "GET") && (request.path() == "/env") {
        print_env(&request, &mut response);
    } else if (request.method == "GET") && (request.path() == "/session") {
        show_session(&request, &mut response);
    } else if (request.method == "POST") && (request.path() == "/session") {
        set_session(&request, &mut response);
    } else if (request.method == "GET") && (request.path().starts_with("/to/")) {
        redirect(&request, &mut response);
    } else if (request.method == "POST") && (request.path() == "/save") {
        shorten(&request, &mut response);
    } else if (request.method == "GET") && (request.path() == "/list") {
        list(&request, &mut response);
    } else if (request.method == "POST") && (request.path() == "/forget") {
        forget(&request, &mut response);
    } else {
        not_found(&mut response);
    }

    response.default_session(&request);
    response.send();
}

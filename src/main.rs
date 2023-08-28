mod token;
mod cgi;

use std::env;
use cgi::{Request, Response};

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

fn show_session(request: &Request, response: &mut Response) {
    response.default_session(&request);

    response.add_header("Content-Type", "text/javascript");
    response.body = format!(r#"{{"session": "{}"}}"#, response.session.as_ref().unwrap());
}

fn set_session(request: &Request, response: &mut Response) {
    match &request.body {
        Some(body) => {
            if body.content_type == "text/javascript" {
                // parse JSON here
                // {"session": "1234"}
                //              ^-- char 14 
                let session = &body.data[14..(body.data.len() - 2)];
                response.set_session(Some(session));
            } else {
                response.status = 400;
                response.body = String::from("Invalid content type");
            }
        },
        None => {
            response.status = 400;
            response.body = String::from("No session provided");
        }
    }
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
    } else {
        not_found(&mut response);
    }


    response.default_session(&request);
    response.send();
}

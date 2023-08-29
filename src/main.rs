mod token;
mod cgi;
mod store;

use std::env;
use serde::{Deserialize, Serialize};

use cgi::{Request, Response};

fn print_env(request: &Request, response: &mut Response) {
    response.status = 200;
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

fn get_db(response: &mut Response) -> Option<rusqlite::Connection> {
    match store::open() {
        Ok(db) => Some(db),
        Err(err) => {
            response.status = 500;
            response.body = String::from(format!("Failed to open database: {}", err));
            None
        }
    }
}

#[derive(Serialize)]
struct ShowSessionResp {
    session: String,
}

fn show_session(_request: &Request, response: &mut Response) {
    response.status = 200;
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
            response.status = 200;
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

#[derive(Deserialize)]
struct ShortenReq {
    url: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct ShortenResp {
    success: bool,
    token: String,
    requested: String,
    url: String,
}

fn shorten(request: &Request, response: &mut Response) {
    let shorten = request.json::<ShortenReq>();
    let shorten = match shorten {
        Ok(shorten) => shorten,
        Err(err) => {
            response.status = 400;
            response.body = String::from(format!("Invalid request: {}", err));
            return;
        }
    };
    let token = match shorten.token {
        Some(token) => if token.len() > 5 { token } else { token::generate(token::TokenType::URL) },
        None => token::generate(token::TokenType::URL),
    };

    let db = match get_db(response) {
        Some(db) => db,
        None => return,
    };

    // start by just trying to use the token as-is, given some size constraints
    let mut token_attempt = token.to_owned();
    if token_attempt.len() > 32 {
        token_attempt.truncate(32);
    } else if token_attempt.len() < 5 {
        token_attempt = token::extend(&token_attempt);
    }

    // token might already be in use; if so, just keep trying with different tokens
    loop {
        match store::create_link(
            &db, &token_attempt, &shorten.url, &response.session.as_ref().unwrap()
        ) {
            Ok(_) => {
                response.status = 200;    
                response.json(&ShortenResp {
                    success: true,
                    token: token_attempt,
                    requested: token,
                    url: shorten.url,
                });
                break;
            },
            Err(err) => {
                response.status = 500;

                let sqlite_err = err.sqlite_error();
                match sqlite_err {
                    Some(sqlite_err) => {
                        if !(rusqlite::ErrorCode::ConstraintViolation == sqlite_err.code) {
                            response.body = String::from(format!("SQL Error creating link: {:?}", sqlite_err));
                            break;
                        }
                    },
                    None => {
                        response.body = String::from(format!("Failed to create link: {}", err));
                    }
                }
            }
        }

        token_attempt = token::extend(&token);
    }
}

fn redirect(request: &Request, response: &mut Response) {
    let token = request.path().trim_start_matches("/to/");
    if token.len() < 5 {
        return not_found(response);
    }

    let db = match get_db(response) {
        Some(db) => db,
        None => return,
    };

    match store::get_link(&db, token) {
        Ok(url) => {
            match url {
                Some(url) => {
                    response.status = 308;
                    response.add_header("Location", &url);
                },
                None => {
                    response.status = 404;
                    response.add_header("Content-Type", "text/plain");
                    response.body = String::from("Not found");
                }
            }
        },
        Err(err) => {
            response.status = 500;
            response.body = String::from(format!("Failed to get link: {}", err));
        }
    }
}

#[derive(Serialize)]
struct ListResp {
    links: Vec<store::Link>,
}

fn list(_request: &Request, response: &mut Response) {
    let db = match get_db(response) {
        Some(db) => db,
        None => return,
    };

    let links = match store::list_links(&db, &response.session.as_ref().unwrap()) {
        Ok(links) => links,
        Err(err) => {
            response.status = 500;
            response.body = String::from(format!("Failed to list links: {}", err));
            return;
        }
    };

    response.status = 200;
    response.json(&ListResp {
        links: links,
    });
}

#[derive(Deserialize)]
struct ForgetReq {
    token: String,
}

#[derive(Serialize)]
struct ForgetResp {
    success: bool,
    token: String,
}

fn forget(request: &Request, response: &mut Response) {
    let forget = request.json::<ForgetReq>();
    let token = match forget {
        Ok(forget) => forget.token,
        Err(err) => {
            response.status = 400;
            response.body = String::from(format!("Invalid request: {}", err));
            return;
        }
    };

    let db = match get_db(response) {
        Some(db) => db,
        None => return,
    };

    let session = response.session.as_ref().unwrap();

    match store::delete_link(&db, &token, &session) {
        Ok(num) => {
            match num {
                0 => {
                    response.status = 404;
                    response.json(&ForgetResp {
                        success: false,
                        token,
                    });
                },
                _ => {
                    response.status = 200;
                    response.json(&ForgetResp {
                        success: true,
                        token,
                    });
                }
            }
        },
        Err(err) => {
            response.status = 500;
            response.body = String::from(format!("Failed to forget link: {}", err));
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

    // our default response is a 404
    not_found(&mut response);

    // init session, either from the request(login), or from cookies, or create a new one
    if (request.method == "POST") && (request.path() == "/session") {
        set_session(&request, &mut response);
    } else {
        response.default_session(&request);
    }
    // now, `response.session` is always set

    // process our routes (this is a very simple router)
    if (request.method == "GET") && (request.path() == "/env") {
        print_env(&request, &mut response);
    } else if (request.method == "GET") && (request.path() == "/session") {
        show_session(&request, &mut response);
    } else if (request.method == "GET") && (request.path().starts_with("/to/")) {
        redirect(&request, &mut response);
    } else if (request.method == "POST") && (request.path() == "/save") {
        shorten(&request, &mut response);
    } else if (request.method == "GET") && (request.path() == "/list") {
        list(&request, &mut response);
    } else if (request.method == "POST") && (request.path() == "/forget") {
        forget(&request, &mut response);
    }

    response.send();
}

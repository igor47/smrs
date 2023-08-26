use std::env;

struct Request {
    method: String,
    uri: String,
    headers: Vec<Header>,
    body: String,
}

struct Header {
    key: &'static str,
    value: &'static str,
}

struct Response {
    status: u16,
    headers: Vec<Header>,
    body: String,
}

fn respond(response: &Response) {
    println!("Status: {0}", response.status);
    for header in &response.headers {
        println!("{0}: {1}", header.key, header.value);
    }
    println!("\n{0}", response.body);
}

fn parse_request() -> Request {
    let request = Request {
        method: env::var("REQUEST_METHOD").unwrap(),
        uri: env::var("REQUEST_URI").unwrap(),
        headers: vec![],
        body: String::new(),
    };

    request
}

fn print_env(request: &Request, response: &mut Response) {
    response.headers.push(Header { key: "Content-Type", value: "text/plain" });

    for (key, value) in env::vars() {
        response.body.push_str(&format!("{0}: {1}\n", key, value));
    }

    response.body.push_str("\nRequest info:\n");
    response.body.push_str(&format!("Method: {0}\n", request.method));
    response.body.push_str(&format!("URI: {0}\n", request.uri));
    response.body.push_str(&format!("headers: {0}\n", request.headers.len()));
    response.body.push_str(&format!("body: {0}\n", request.body))
}

fn main() {
    let request = parse_request();
    let mut response = Response {
        status: 200,
        headers: vec![Header { key: "SMRS-Version", value: "0.0.1" }],
        body: String::new(),
    };

    print_env(&request, &mut response);

    respond(&response);
}

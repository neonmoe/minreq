use std::thread;
use std::sync::{Once, ONCE_INIT};
use tiny_http::{Method, Response, Server};

static INIT: Once = ONCE_INIT;

pub fn setup() {
    INIT.call_once(|| {
        thread::spawn(|| {
            let server = Server::http("0.0.0.0:35562").unwrap();
            let mut list = Vec::new();
            for mut request in server.incoming_requests() {
                if *request.method() == Method::Get && &*request.url() == "/boop" {
                    request.respond(Response::from_string("beep")).ok();
                } else if *request.method() == Method::Get && &*request.url() == "/list" {
                    request
                        .respond(Response::from_string(format!("{:?}", list)))
                        .ok();
                } else if *request.method() == Method::Post && &*request.url() == "/insert" {
                    let mut content = String::new();
                    if let Ok(_) = request.as_reader().read_to_string(&mut content) {
                        list.push(content);
                    }
                    request
                        .respond(Response::from_string("ok").with_status_code(201))
                        .ok();
                }
            }
        });
    });
}

pub fn url(req: &str) -> String {
    format!("http://0.0.0.0:35562{}", req)
}

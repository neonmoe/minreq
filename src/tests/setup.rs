use std::thread;
use std::io::Error;
use std::sync::{Once, ONCE_INIT};
use std::time::Duration;
use std::sync::Arc;
use tiny_http::{Method, Response, Server};
use http;

static INIT: Once = ONCE_INIT;

pub(crate) fn setup() {
    INIT.call_once(|| {
        let server = Arc::new(Server::http("0.0.0.0:35562").unwrap());
        for _ in 0..4 {
            let server = server.clone();

            thread::spawn(move || loop {
                let mut request = server.recv().unwrap();
                let mut content = String::new();
                request.as_reader().read_to_string(&mut content).ok();

                let url = String::from(request.url());
                match request.method() {
                    &Method::Get if url == "/slow_a" => {
                        thread::sleep(Duration::from_secs(2));
                        let response = Response::from_string(format!("j: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Get if url == "/a" => {
                        let response = Response::from_string(format!("j: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Head if url == "/b" => {
                        request.respond(Response::empty(420)).ok();
                    }
                    &Method::Post if url == "/c" => {
                        let response = Response::from_string(format!("l: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Put if url == "/d" => {
                        let response = Response::from_string(format!("m: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Delete if url == "/e" => {
                        let response = Response::from_string(format!("n: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Trace if url == "/f" => {
                        let response = Response::from_string(format!("o: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Options if url == "/g" => {
                        let response = Response::from_string(format!("p: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Connect if url == "/h" => {
                        let response = Response::from_string(format!("q: {}", content));
                        request.respond(response).ok();
                    }
                    &Method::Patch if url == "/i" => {
                        let response = Response::from_string(format!("r: {}", content));
                        request.respond(response).ok();
                    }

                    _ => {
                        request
                            .respond(Response::from_string("Not Found").with_status_code(404))
                            .ok();
                    }
                }
            });
        }
    });
}

pub(crate) fn url(req: &str) -> String {
    format!("http://0.0.0.0:35562{}", req)
}

pub(crate) fn get_body(request: Result<http::Response, Error>) -> String {
    match request {
        Ok(response) => String::from(response.body.trim()),
        Err(err) => {
            println!("\n[ERROR]: {}\n", err);
            String::new()
        }
    }
}

pub(crate) fn get_status_code(request: Result<http::Response, Error>) -> i32 {
    match request {
        Ok(response) => response.status_code,
        Err(err) => {
            println!("\n[ERROR]: {}\n", err);
            -1
        }
    }
}

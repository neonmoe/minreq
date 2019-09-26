extern crate minreq;
extern crate tiny_http;
use self::minreq::MinreqError;
use self::tiny_http::{Header, Method, Response, Server};
use std::sync::Arc;
use std::sync::{Once, ONCE_INIT};
use std::thread;
use std::time::Duration;

static INIT: Once = ONCE_INIT;

pub fn setup() {
    INIT.call_once(|| {
        let server = Arc::new(Server::http("localhost:35562").unwrap());
        for _ in 0..4 {
            let server = server.clone();

            thread::spawn(move || loop {
                let mut request = server.recv().unwrap();
                let mut content = String::new();
                request.as_reader().read_to_string(&mut content).ok();
                let headers = Vec::from(request.headers());

                let url = String::from(request.url());
                match request.method() {
                    Method::Get if url == "/header_pong" => {
                        for header in headers {
                            if header.field.as_str() == "Ping" {
                                let response = Response::from_string(format!("{}", header.value));
                                request.respond(response).ok();
                                return;
                            }
                        }
                        request.respond(Response::from_string("No header!")).ok();
                    }

                    Method::Get if url == "/slow_a" => {
                        thread::sleep(Duration::from_secs(2));
                        let response = Response::from_string(format!("j: {}", content));
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/a" => {
                        let response = Response::from_string(format!("j: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Post if url == "/a" => {
                        let response = Response::from_string("POST to /a is not valid.");
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/redirect" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(&b"Location"[..], &b"http://localhost:35562/a"[..])
                                .unwrap(),
                        );
                        request.respond(response).ok();
                    }
                    Method::Post if url == "/redirect" => {
                        let response = Response::empty(303).with_header(
                            Header::from_bytes(&b"Location"[..], &b"http://localhost:35562/a"[..])
                                .unwrap(),
                        );
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/infiniteredirect" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(
                                &b"Location"[..],
                                &b"http://localhost:35562/redirectpong"[..],
                            )
                            .unwrap(),
                        );
                        request.respond(response).ok();
                    }
                    Method::Get if url == "/redirectpong" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(
                                &b"Location"[..],
                                &b"http://localhost:35562/infiniteredirect"[..],
                            )
                            .unwrap(),
                        );
                        request.respond(response).ok();
                    }

                    Method::Post if url == "/echo" => {
                        request.respond(Response::from_string(content)).ok();
                    }

                    Method::Head if url == "/b" => {
                        request.respond(Response::empty(418)).ok();
                    }
                    Method::Post if url == "/c" => {
                        let response = Response::from_string(format!("l: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Put if url == "/d" => {
                        let response = Response::from_string(format!("m: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Delete if url == "/e" => {
                        let response = Response::from_string(format!("n: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Trace if url == "/f" => {
                        let response = Response::from_string(format!("o: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Options if url == "/g" => {
                        let response = Response::from_string(format!("p: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Connect if url == "/h" => {
                        let response = Response::from_string(format!("q: {}", content));
                        request.respond(response).ok();
                    }
                    Method::Patch if url == "/i" => {
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

pub fn url(req: &str) -> String {
    format!("http://localhost:35562{}", req)
}

pub fn get_body(request: Result<minreq::Response, MinreqError>) -> String {
    match request {
        Ok(response) => match response.as_str() {
            Ok(str) => String::from(str),
            Err(err) => {
                println!("\n[ERROR]: {}\n", err);
                String::new()
            }
        },
        Err(err) => {
            println!("\n[ERROR]: {}\n", err);
            String::new()
        }
    }
}

pub fn get_status_code(request: Result<minreq::Response, MinreqError>) -> i32 {
    match request {
        Ok(response) => response.status_code,
        Err(err) => {
            println!("\n[ERROR]: {}\n", err);
            -1
        }
    }
}

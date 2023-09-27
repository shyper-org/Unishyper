/// Example is derived from tiny-http example
/// https://github.com/tiny-http/tiny-http/blob/master/examples/hello-world.rs
#[cfg(target_os = "shyper")]
use unishyper as _;

use std::sync::Arc;

fn main() {
    let server = Arc::new(tiny_http::Server::http("0.0.0.0:4444").unwrap());

    let heart = vec![240, 159, 146, 151];
    let text = format!(
        "Hello from Unishyper {}",
        String::from_utf8(heart).unwrap_or_default()
    );
    for request in server.incoming_requests() {
        println!(
            "received request! method: {:?}, url: {:?}, headers:",
            request.method(),
            request.url()
        );
        for h in request.headers() {
            println!("\t[{}]", h);
        }

        let response = tiny_http::Response::from_string(text.clone());
        request.respond(response).expect("Responded");
    }
}

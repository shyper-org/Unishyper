/// Example is derived from tiny-http example
/// https://github.com/tiny-http/tiny-http/blob/master/examples/hello-world.rs
#[cfg(target_os = "shyper")]
use unishyper as _;

fn main() {
	println!("Hello!");
	
	let heart = vec![240, 159, 146, 151];
	let text = format!(
		"Hello from Unishyper {}",
		String::from_utf8(heart).unwrap_or_default()
	);

	// let listener = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
    // // let stream = listener.incoming().next().unwrap().unwrap();
    // // println!("Connection established with {:?}!", stream.peer_addr().unwrap());
	// let addr = listener.local_addr().unwrap();
	// println!("test bind on {}", addr);

	let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
	println!("Now listening on {}", server.server_addr());

	for request in server.incoming_requests() {
		println!(
			"received request! method: {:?}, url: {:?}, headers: {:?}",
			request.method(),
			request.url(),
			request.headers()
		);

		let response = tiny_http::Response::from_string(text.clone());
		request.respond(response).expect("Responded");
	}
}

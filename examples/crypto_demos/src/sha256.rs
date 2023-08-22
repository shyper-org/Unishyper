use ring::digest::{Context, Digest, SHA256};
use data_encoding::HEXUPPER;

use unishyper::*;
use unishyper::shyperstd as std;

use std::io::{BufReader, Read, Write};

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, &'static str> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn sha256_test() {
    let input = "hello".as_bytes();

    let reader = BufReader::new(input);
    let digest = sha256_digest(reader).expect("sha256_digest");

    println!("SHA-256 digest is {}", HEXUPPER.encode(digest.as_ref()));
}
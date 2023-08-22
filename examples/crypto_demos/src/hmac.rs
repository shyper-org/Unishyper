use unishyper::println;

use ring::{digest, hmac, rand};
use ring::error::Unspecified;
use ring::rand::SecureRandom;

pub fn hmac_sign_and_verify() {
    let rng = rand::SystemRandom::new();
    let key =
        hmac::Key::generate(hmac::HMAC_SHA256, &rng).expect("rand::generate key value failed");

    let message = "Legitimate and important message.";
    let message_wrong = "Legitimate and important message!!!";

    let tag = hmac::sign(&key, message.as_bytes());

    match hmac::verify(&key, message.as_bytes(), tag.as_ref()) {
        Ok(()) => {
            println!("Ok!!! Msg \"{}\" verified success!", message);
        }
        Err(_) => {
            println!("Err!!! Msg \"{}\" verified success!", message);
        }
    }

    match hmac::verify(&key, message_wrong.as_bytes(), tag.as_ref()) {
        Ok(()) => {
            println!(
                "Err!!! Wrong \"{}\" verified failed, right msg {}",
                message_wrong, message
            );
        }
        Err(_) => {
            println!(
                "Ok!!! Wrong \"{}\" verified success, right msg {}",
                message_wrong, message
            );
        }
    }
}

// Using the one-shot API.
pub fn hmac_sign_and_verify_one_shot() {
    let msg = "hello, world";

    // The sender generates a secure key value and signs the message with it.
    // Note that in a real protocol, a key agreement protocol would be used to
    // derive `key_value`.
    let rng = rand::SystemRandom::new();
    let key_value: [u8; digest::SHA256_OUTPUT_LEN] = rand::generate(&rng)
        .expect("rand::generate key value failed")
        .expose();

    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key_value.as_ref());
    let tag = hmac::sign(&s_key, msg.as_bytes());

    // The receiver (somehow!) knows the key value, and uses it to verify the
    // integrity of the message.
    let v_key = hmac::Key::new(hmac::HMAC_SHA256, key_value.as_ref());
    match hmac::verify(&v_key, msg.as_bytes(), tag.as_ref()) {
        Ok(()) => {
            println!("Ok!!! Msg \"{msg}\" verified success!");
        }
        Err(_) => {
            println!("Err!!! Msg \"{msg}\" verified success!");
        }
    }

    //Ok::<(), ring::error::Unspecified>(())
}

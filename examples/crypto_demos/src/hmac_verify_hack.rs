use unishyper::*;
use unishyper::shyperstd as std;

use core::time::Duration;

use ring::{digest, hmac, rand};
use ring::error::Unspecified;
use ring::rand::SecureRandom;

static mut SECURE_KEY: [u8; digest::SHA256_OUTPUT_LEN] = [0; digest::SHA256_OUTPUT_LEN];

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

pub fn hmac_sign_verify() {
    let msg = "hello Unishyper";

    let rng = rand::SystemRandom::new();
    let generated_key_value: [u8; digest::SHA256_OUTPUT_LEN] = rand::generate(&rng)
        .expect("rand::generate key value failed")
        .expose();
    unsafe {
        SECURE_KEY.copy_from_slice(&generated_key_value);
        println!("{:?} ", SECURE_KEY);
    }

    let key_value_ref = unsafe { SECURE_KEY.as_ref() };

    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key_value_ref);
    let tag = hmac::sign(&s_key, msg.as_bytes());

    loop {
        let v_key = hmac::Key::new(hmac::HMAC_SHA256, key_value_ref);
        match hmac::verify(&v_key, msg.as_bytes(), tag.as_ref()) {
            Ok(()) => {
                println!("Ok!!! Msg \"{msg}\" verified success!");
            }
            Err(_) => {
                println!("Err!!! Msg \"{msg}\" verified success!");
            }
        }
        std::thread::sleep(Duration::new(2, 0));
    }
}

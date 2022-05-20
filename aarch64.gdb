target remote 127.0.0.1:1234
file target/aarch64shyper/release/rust-shyper-os
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on
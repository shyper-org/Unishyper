target remote 127.0.0.1:1234
file examples/hello_world/target/aarch64-unknown-shyper/release/hello_world
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on

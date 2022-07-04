target remote 127.0.0.1:1234
file examples/net_demo/target/aarch64shyper/release/net_demo
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on
break handle_interrupt
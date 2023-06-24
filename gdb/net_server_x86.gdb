target remote 127.0.0.1:1234
file examples/net_demo/target/x86_64qemu/release/server-bw
break entry
set confirm off
display/i $pc
set print asm-demangle on
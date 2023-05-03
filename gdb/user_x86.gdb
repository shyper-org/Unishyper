target remote 127.0.0.1:1234
file examples/user/target/x86_64qemu/release/user
break entry
set confirm off
display/i $pc
set print asm-demangle on

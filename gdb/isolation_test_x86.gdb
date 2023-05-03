target remote 127.0.0.1:1234
file examples/isolation_test/target/x86_64qemu/release/isolation_test
break entry
set confirm off
display/i $pc
set print asm-demangle on

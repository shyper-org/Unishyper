target remote 127.0.0.1:1234
file examples/httpd/target/x86_64-unknown-shyper/release/httpd
break entry
set confirm off
display/i $pc
set print asm-demangle on
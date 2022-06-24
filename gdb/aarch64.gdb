target remote 127.0.0.1:1234
file user/target/aarch64shyper/release/user
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on
break set_cpu_context
break save_context
break thread_yield
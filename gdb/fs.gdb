target remote 127.0.0.1:1234
file examples/fs_demo/target/aarch64qemu/release/fs_demo
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on

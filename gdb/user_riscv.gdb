target remote 127.0.0.1:1234
file examples/user/target/riscv64qemu/release/user
set confirm off
display/i $pc
set print asm-demangle on

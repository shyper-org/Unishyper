target remote 127.0.0.1:1234
file examples/user/target/aarch64shyper/release/user
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on
break current_el_sp0_synchronous
break current_el_spx_synchronous
break current_el_sp0_irq
break current_el_spx_irq
break save_context
break set_cpu_context

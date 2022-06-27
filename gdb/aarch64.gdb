target remote 127.0.0.1:1234
file user/target/aarch64shyper/release/user
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on
break pop_context
break pop_context_first
break main
break set_cpu_context
break save_context
break thread_yield
break current_el_sp0_synchronous
break current_el_spx_synchronous
break current_el_sp0_irq
break current_el_spx_irq
break test_yield_thread_1
break test_yield_thread_2
break vectors

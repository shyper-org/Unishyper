ENTRY(_start)

KERNEL_ENTRY = 0xFFFFFF0000000000;

SECTIONS
{
    . = KERNEL_ENTRY;
    .init : {
        *(.text.start)
        *(.data.start)
    }
    
    .text : {
        *(.text*)
    }
    . = ALIGN(4096);
    .rodata : {
        *(.rodata*)
    }
    . = ALIGN(4096);
    .got : {
        *(.got*)
    }
    . = ALIGN(4096);
    .data : {
        *(.data*)
    }
    . = ALIGN(4096);
    BSS_START = .;
    .bss : {
        *(.bss*)
    }
    . = ALIGN(4096);
    BSS_END = .;
    .stack (NOLOAD) : {
        *(.stack)
    }
    KERNEL_END = .;
}

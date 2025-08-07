OUTPUT_FORMAT("elf64-x86-64")
ENTRY(_start)
SECTIONS {
    . = 0x100000;
    . = ALIGN(4096);
    .text   : {
        *(.text .text.*)
    }
    . = ALIGN(4096);
    .rodata : {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4096);
    .data   : {
        *(.data .data.*)
    }
    . = ALIGN(4096);
    .bss    : {
        *(COMMON) *(.bss .bss.*)
    }
}

SECTIONS {
    . = 0x0;
    .text : AT ( 0x80000000 ) { *(.text) }
    .rodata : AT ( 0x80000000 + SIZEOF(.text) ) { *(.rodata) }
}

SECTIONS {
    . = 0x0;
    .text : AT ( 0x80000000 ) {
        *(.vector)
        *(.text)
    }
}

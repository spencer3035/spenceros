ENTRY(_start)

SECTIONS {
    . = 0x8000;
    /* MARKER USED FOR TESTS, DO NOT REMOVE */
    _start_address = .;

    .start :
    {
        *(.start)
    }
    .text :
    {
        *(.text .text.*)
    }
    .bss : {
        *(.bss .bss.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.data .data.*)
    }
    .eh_frame : {
        *(.eh_frame .eh_frame.*)
    }
    .eh_frame_hdr : {
        *(.eh_frame_hdr .eh_frame_hdr.*)
    }

    _second_stage_end = .;
    . = 0x10000 - 0x2;
    .end_marker :
    {
        SHORT(0xdead)
    }
    /* MARKER USED FOR TESTS, DO NOT REMOVE */
    _end_address = .;
}

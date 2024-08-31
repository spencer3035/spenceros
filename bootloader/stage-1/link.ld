ENTRY(_start)

SECTIONS {
    . = 0x7c00 + 0x200;

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
    . = 0x7c00 + 0x2200 - 0x2;
    .end_marker :
    {
        SHORT(0xdead)
    }
}

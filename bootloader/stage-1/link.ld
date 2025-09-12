ENTRY(_start)

SECTIONS {
    /* STAGE_0_START + 0x200 */
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
    /* STAGE_0_START + 0x200 + 0x200 * STAGE_1_SECTIONS - 0x2 */
    . = 0x7c00 + 0x200 + 0x200 * 0x30 - 0x2;
    .end_marker :
    {
        SHORT(0xdead)
    }
}

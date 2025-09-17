ENTRY(_start)

SECTIONS {
    . = 0x7c00;
    /* MARKER USED FOR TESTS, DO NOT REMOVE */
    _start_address = .;
    .boot :
    {
        *(.boot .boot.*)
    }
    .text :
    {
        *(.text .text.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }
    _mbr_end = .;

    . = 0x7c00 + 446;
    _partition_table = .;
    .partition_table :
    {
        /* partition table entry 0 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 1 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 2 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 3 */
        QUAD(0)
        QUAD(0)
    }

    . = 0x7e00 - 2;

    .magic_number :
    {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }

    /* MARKER USED FOR TESTS, DO NOT REMOVE */
    _end_address = .;
}

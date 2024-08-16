.section .boot, "awx"
.global _start
.code16

# Initialize the stack and jump to rust
_start:
  # Some BIOS' may load us at 0x0000:0x7C00 while other may load us at 0x07C0:0x0000.
  # Do a far jump to fix this issue, and reload CS to 0x0000.
  # How do we do this?
  #jmp 0x000:_flush_cs
#_flush_cs:
  # zero segment registers
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov ss, ax
  mov fs, ax
  mov gs, ax

  # clear the direction flag (e.g. go forward in memory when using
  # instructions like lodsb)
  cld
  # initialize stack
  mov sp, 0x7c00

rust:
  # push disk number as argument
  push dx
  call main

spin:
  hlt
  jmp spin

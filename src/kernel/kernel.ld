ENTRY(boot) /* kenel start is "boot" symbol name */

/*
  ALIGN(4): 4byte alignment
  .: current address

  formats:
  <section name> :{
    symbol_name = address
  }
*/

SECTIONS {
    . = 0x80200000; /* Base address determined by OpenSBI used in riscv's qemu */
    __kernel_base = .;
    /* text section: Code for execution */
    .text :{
        KEEP(*(.text.boot));
        *(.text .text.*);
    }
    /* rodata section: Readonly Data: const variables */
    .rodata : ALIGN(4) {
        *(.rodata .rodata.*);
    }
    /* data section: Read/Write variables */
    .data : ALIGN(4) {
        *(.data .data.*);
    }
    /* bss section: 0 fill global, static variables. Allocation at runtime. */
    .bss : ALIGN(4) {
        __bss = .;
        *(.bss .bss.* .sbss .sbss.*);
        __bss_end = .;
    }

    . = ALIGN(4);
    . += 128 * 1024; /* 128KB */
    __stack_top = .;

    . = ALIGN(4096);
    __free_ram = .;
    . += 64 * 1024 * 1024; /* 64MB */
    __free_ram_end = .;
}

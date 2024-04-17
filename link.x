/* Define the start address of RAM and set the kernel's start address 32MB (0x2000000) into the RAM. */
_RAM_START = 0x80000000;
_KERNEL_START = _RAM_START + 0x2000000;

/* Set the physical and virtual addresses for the stack. The physical stack starts 16MB beyond the kernel start,
   and the virtual stack is set to a high address in the virtual address space. */
_PHYSICAL_STACK = _KERNEL_START + 16M;
_VIRTUAL_STACK = 0xffffffffbfc00000;

/* Define the length of the stack as 2MB. */
_STACK_LEN = 2M;
/* Set the virtual address where the kernel code will be mapped. */
_KERNEL_CODE_VIRTUAL = 0xffffffffc0000000;

/* Calculate the start of the virtual RAM for the kernel based on the kernel's virtual code start address and
   the offset from RAM start to kernel start. */
_KERNEL_VIRTUAL_RAM_START = _KERNEL_CODE_VIRTUAL + (_KERNEL_START - _RAM_START);

/* Begin section definitions. */
SECTIONS
{
  /* Set the location of the start of the kernel. */
  . = _KERNEL_START;

  /* Define a section named .physical_boot.text for boot code in physical memory.
     This section collects all initialization code segments from the input files.
     It marks the start and end of this boot code section, enabling precise control
     over the execution flow during system startup. */
  
  .physical_boot.text : {
      /* Mark the start address of the boot code section. */
      _boot_start = .;
      
      /* Include all .init sections (initialization code) from the input files
         into this boot code section. */
      *(.init)
      
      /* Mark the end address of the boot code section. The ABSOLUTE keyword
         ensures this address is unaffected by any subsequent relocations. */
      _boot_end = ABSOLUTE(.);
  }

    /* Set the location counter (.) to the start of the virtual stack. */
    . = _VIRTUAL_STACK;

    /* Define the .stack section, marking it as NOLOAD since it doesn't need to be loaded from the binary.
       It's placed at the physical stack address. */
    .stack (NOLOAD) : AT(_PHYSICAL_STACK)
    {
        _estack = .; /* Mark the current location as the start of the stack. */
        . += _STACK_LEN; /* Reserve space for the stack by advancing the location counter. */
        _sstack = .; /* Mark the end of the stack. */
    }

    /* Calculate the virtual address for the start of the kernel code, taking into account
       the end of the bootloader (_boot_end) and the offset from the kernel start. */
    _KERNEL_VIRTUAL_CODE_START = _KERNEL_VIRTUAL_RAM_START + (_boot_end - _KERNEL_START);

    /* Calculate the offset for virtual addresses in the kernel. */
    HIDDEN(_KERNEL_VA_CODE_OFFSET = _KERNEL_VIRTUAL_CODE_START - _boot_end);
    . = _KERNEL_VIRTUAL_CODE_START; /* Set the location counter to the start of the kernel virtual code. */

    /* Define the .text section for executable code. It uses the calculated VA offset for its physical address (AT()). */
    .text : AT(ADDR(.text) - _KERNEL_VA_CODE_OFFSET) {
        *(.text.abort); /* Include abort handler code. */
        *(text .text.*) /* Include all other .text sections. */
    }

    /* Define the .rodata (read-only data) section, aligning it to 4 bytes. */
    .rodata ALIGN(4) : AT(ADDR(.rodata) - _KERNEL_VA_CODE_OFFSET) {
        *(.srodata .srodata.*); /* Include small read-only data. */
        *(.rodata .rodata.*); /* Include all other read-only data. */

        /* Align the end of the .rodata section to 4 bytes. */
        . = ALIGN(4);
    }

    /* Define the .data section for initialized data, aligning it to 8 bytes. */
    .data ALIGN(8) : AT(ADDR(.data) - _KERNEL_VA_CODE_OFFSET) {
        _sidata = LOADADDR(.data); /* Store the load address of the .data section. */
        _sdata = .; /* Mark the start of the .data section. */
        PROVIDE(__global_pointer$ = . + 0x800); /* Set the global pointer to an offset within the .data section. */
        *(.sdata .sdata.* .sdata2 .sdata2.*); /* Include specific data sections. */
        *(.data .data.*); /* Include all other data sections. */
        . = ALIGN(8); /* Align the end of the .data section to 8 bytes. */
        _edata = .; /* Mark the end of the .data section. */
    }

    /* Define the .bss section for uninitialized data, aligning it to 8 bytes and marking it as NOLOAD. */
    .bss ALIGN(8) (NOLOAD) : AT(ADDR(.data) - _KERNEL_VA_CODE_OFFSET) {
        _sbss = .; /* Mark the start of the .bss section. */
        *(.sbss .sbss.* .bss .bss.*); /* Include specific BSS sections. */
        . = ALIGN(8); /* Align the end of the .bss section to 8 bytes. */
        _ebss = .; /* Mark the end of the .bss section. */
    }
}

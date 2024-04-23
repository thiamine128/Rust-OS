OUTPUT_ARCH(mips)

ENTRY(_start)

SECTIONS {
	. = 0x80000000;
	.tlb_miss_entry : {
		*(.text.tlb_miss_entry)
	}
	. = 0x80000180;
	.exc_gen_entry : {
		*(.text.exc_gen_entry)
	}
	. = 0x80020000;

	
	.text : {
		text_start = .;
		*(.text.entry)
        . = ALIGN(4K);
        *(.text.ebase)
        *(.text .text.*)
        . = ALIGN(4K);
		text_end = .;
	}
	
	.rodata : {
        rodata_start = .;
        *(.rodata .rodata.*)
        *(.dtb)
        . = ALIGN(4K);
        rodata_end = .;
    }
	
	.data : {
        data_start = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        data_end = .;
    }

    .bss : {
        bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        bss_end = .;
    }

	. = 0x80400000;
	end = .;

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}


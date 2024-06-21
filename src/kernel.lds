OUTPUT_ARCH(mips)

ENTRY(_start)

SECTIONS {
    . = 0x80010000;
	.tlb_miss_entry : {
        exc_entry = .;
		*(.text.tlb_miss_entry)
	}
	. = exc_entry + 0x180;
	.exc_gen_entry :  {
	    *(.text.exc_gen_entry)
	}

    . = 0x80020000;
	.text : {
        *(.text .text.*)
	}
	
	.rodata : {
        rodata_start = .;
        *(.rodata .rodata.*)
        rodata_end = .;
    }
	
	.data : {
        data_start = .;
        *(.data .data.*)
        data_end = .;
    }

    .bss : {
        bss_start = .;
        *(.bss .bss.*)
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


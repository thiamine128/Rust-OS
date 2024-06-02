#include <asm/asm.h>

.section .text.entry
EXPORT(_start)
.set at
.set reorder
    la v0, bss_start
    la v1, bss_end
clear_bss_loop:
    beq v0, v1, clear_bss_done
    sb zero, 0(v0)
    addiu v0, v0, 1
    j clear_bss_loop
clear_bss_done:
    mtc0 zero, CP0_STATUS
    li sp, 0x80400000
    la v0, exc_entry
    mtc0 v0, CP0_EBASE
    jal rust_main
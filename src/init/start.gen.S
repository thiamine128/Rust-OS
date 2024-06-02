# 1 "src/init/start.S"
# 1 "<built-in>"
# 1 "<command-line>"
# 1 "src/init/start.S"
# 1 "./include4asm/asm/asm.h" 1
# 1 "./include4asm/asm/cp0regdef.h" 1
# 2 "./include4asm/asm/asm.h" 2
# 1 "./include4asm/asm/regdef.h" 1
# 3 "./include4asm/asm/asm.h" 2
# 2 "src/init/start.S" 2

.section .text.entry
.globl _start; _start:
.set at
.set reorder
    la $2, bss_start
    la $3, bss_end
clear_bss_loop:
    beq $2, $3, clear_bss_done
    sb $0, 0($2)
    addiu $2, $2, 1
    j clear_bss_loop
clear_bss_done:
    mtc0 $0, $12
    li $29, 0x80400000
    la $2, exc_entry
    mtc0 $2, $15, 1
    jal rust_main

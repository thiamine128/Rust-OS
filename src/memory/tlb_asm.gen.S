# 0 "src/memory/tlb_asm.S"
# 0 "<built-in>"
# 0 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 0 "<command-line>" 2
# 1 "src/memory/tlb_asm.S"
# 1 "./include/asm/asm.h" 1
# 1 "./include/asm/cp0regdef.h" 1
# 2 "./include/asm/asm.h" 2
# 1 "./include/asm/regdef.h" 1
# 3 "./include/asm/asm.h" 2
# 2 "src/memory/tlb_asm.S" 2

.globl tlb_out; .align 2; .type tlb_out, @function; .ent tlb_out; tlb_out: .frame $29, 0, $31
.set noreorder
 mfc0 $8, $10
 mtc0 $4, $10
 nop


 tlbp
 nop

 mfc0 $9, $0
.set reorder
 bltz $9, NO_SUCH_ENTRY
.set noreorder
 mtc0 $0, $10
 mtc0 $0, $2
 mtc0 $0, $3
 nop


 tlbwi
.set reorder

NO_SUCH_ENTRY:
 mtc0 $8, $10
 j $31
.end tlb_out; .size tlb_out, .- tlb_out

.globl do_tlb_refill; .align 2; .type do_tlb_refill, @function; .ent do_tlb_refill; do_tlb_refill: .frame $29, 24, $0
 mfc0 $5, $8
 mfc0 $6, $10
 andi $6, $6, 0xff
.globl do_tlb_refill_call;
do_tlb_refill_call:
 addi $29, $29, -24
 sw $31, 20($29)
 addi $4, $29, 12
 jal _do_tlb_refill
 lw $4, 12($29)
 lw $5, 16($29)
 lw $31, 20($29)
 addi $29, $29, 24
 mtc0 $4, $2
 mtc0 $5, $3
 nop


 tlbwr
 jr $31
.end do_tlb_refill; .size do_tlb_refill, .- do_tlb_refill

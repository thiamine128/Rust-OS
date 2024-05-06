# 0 "src/exception/entry.S"
# 0 "<built-in>"
# 0 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 0 "<command-line>" 2
# 1 "src/exception/entry.S"
# 1 "./include/asm/asm.h" 1
# 1 "./include/asm/cp0regdef.h" 1
# 2 "./include/asm/asm.h" 2
# 1 "./include/asm/regdef.h" 1
# 3 "./include/asm/asm.h" 2
# 2 "src/exception/entry.S" 2
# 1 "./include/stackframe.h" 1
# 3 "src/exception/entry.S" 2

.section .text.tlb_miss_entry
tlb_miss_entry:
 j exc_gen_entry

.section .text.exc_gen_entry
exc_gen_entry:
 SAVE_ALL






 mfc0 $8, $12
 and $8, $8, ~(0x0010 | 0x0002 | 0x0001)
 mtc0 $8, $12

 mfc0 $8, $13
 andi $8, 0x7c
 lw $8, exception_handlers($8)
 jr $8

# 1 "src/env/env_asm.S"
# 1 "<built-in>"
# 1 "<command-line>"
# 1 "src/env/env_asm.S"
# 1 "./include4asm/asm/asm.h" 1
# 1 "./include4asm/asm/cp0regdef.h" 1
# 2 "./include4asm/asm/asm.h" 2
# 1 "./include4asm/asm/regdef.h" 1
# 3 "./include4asm/asm/asm.h" 2
# 2 "src/env/env_asm.S" 2
# 1 "./include4asm/mmu.h" 1
# 3 "src/env/env_asm.S" 2
# 1 "./include4asm/trap.h" 1
# 4 "src/env/env_asm.S" 2
# 1 "./include4asm/kclock.h" 1



# 1 "./include4asm/asm/asm.h" 1
# 5 "./include4asm/kclock.h" 2




.macro RESET_KCLOCK
 li $8, (500000)
 mtc0 $0, $9
 mtc0 $8, $11
.endm
# 5 "src/env/env_asm.S" 2

.text
.globl env_pop_tf; .align 2; .type env_pop_tf, @function; .ent env_pop_tf; env_pop_tf: .frame $29, 0, $31
.set reorder
.set at
 mtc0 $5, $10
 move $29, $4
 RESET_KCLOCK
 j ret_from_exception
.end env_pop_tf; .size env_pop_tf, .- env_pop_tf

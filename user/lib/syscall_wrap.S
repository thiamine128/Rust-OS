#include <asm/asm.h>

LEAF(msyscall)
	syscall
	jr      ra

END(msyscall)

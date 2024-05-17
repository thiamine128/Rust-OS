#ifndef _KCLOCK_H_
#define _KCLOCK_H_

#include <asm/asm.h>

#define TIMER_INTERVAL (500000) // WARNING: DO NOT MODIFY THIS LINE!

// clang-format off
.macro RESET_KCLOCK
	li 	t0, TIMER_INTERVAL
	mtc0	zero, CP0_COUNT
	mtc0	t0, CP0_COMPARE
.endm
// clang-format on
#endif

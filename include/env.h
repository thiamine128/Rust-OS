#ifndef _ENV_H_
#define _ENV_H_

#include <mmu.h>
#include <trap.h>
#include <types.h>
#define LOG2NENV 10
#define NENV (1 << LOG2NENV)
#define ENVX(envid) ((envid) & (NENV - 1))

// All possible values of 'env_status' in 'struct Env'.
#define ENV_FREE 0
#define ENV_RUNNABLE 1
#define ENV_NOT_RUNNABLE 2

struct Env {
	struct Trapframe env_tf;	 // saved context (registers) before switching
	u_int env_id;			 // unique environment identifier
	u_int env_asid;			 // ASID of this env
	u_int env_parent_id;		 // env_id of this env's parent
	u_int env_status;		 // status of this env
	Pde *env_pgdir;			 // page directory
	u_int env_pri;			 // schedule priority

	// Lab 4 IPC
	u_int env_ipc_value;   // the value sent to us
	u_int env_ipc_from;    // envid of the sender
	u_int env_ipc_recving; // whether this env is blocked receiving
	u_int env_ipc_dstva;   // va at which the received page should be mapped
	u_int env_ipc_perm;    // perm in which the received page should be mapped

	// Lab 4 fault handling
	u_int env_user_tlb_mod_entry; // userspace TLB Mod handler

	// Lab 6 scheduler counts
	u_int env_runs; // number of times we've been env_run'ed
};

#endif // !_ENV_H_

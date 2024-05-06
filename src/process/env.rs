use crate::{exception::traps::Trapframe, memory::{mmu::NASID, page_table::PageTable}};


pub const LOG2ENV: usize = 10;
pub const NENV: usize = 1 << LOG2ENV;

#[inline]
pub fn envx(envid: usize) -> usize {
    envid & (NENV - 1)
}

pub struct EnvLink(usize);

pub struct EnvQueue {
    head: EnvLink
}

pub struct Env {
    env_tf: *mut Trapframe,
    env_link: EnvLink,
    env_id: usize,
    env_parent_id: usize,
    env_status: usize,
    env_pgdir: *mut PageTable,
    env_sched_link: EnvLink,
    env_pri: usize
}

pub struct EnvManager {
    envs: [Env; NENV],
    asid_bitmap: [usize; NASID / 32],
    base_pgdir: *mut PageTable,
    curenv: *mut Env
}
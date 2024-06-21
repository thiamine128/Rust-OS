use core::{borrow::BorrowMut, ffi::CStr, mem::{self, size_of}, ptr::write_volatile, slice, usize};


use crate::{device::DeviceManager, env::{env_destroy, env_sched, envid2ind, get_cur_env_id, EnvID}, err::Error, exception::traps::Trapframe, memory::{frame::frame_alloc, mmu::{PhysAddr, VirtAddr, KSTACKTOP, PTE_V, UTEMP, UTOP}, shm::{shm_at, shm_dt, shm_get, shm_rmid, ShmCtl}}, print::{printcharc, scancharc}, println, try_or_return};

use super::{sem::SEM_MAMANER, EnvStatus, ENV_MANAGER};

/// syscall id enum
#[repr(usize)]
pub enum SyscallID {
	Putchar,
	PrintCons,
	GetEnvID,
	Yield,
	EnvDestroy,
	SetTlbModEntry,
	MemAlloc,
	MemMap,
	MemUnmap,
	Exofork,
	SetEnvStatus,
	SetTrapframe,
	Panic,
	IpcTrySend,
	IpcRecv,
	CGetC,
	WriteDev,
	ReadDev,
	ShmGet,
	ShmAt,
	ShmDt,
	ShmCtl,
	SemOpen,
	SemWait,
	SemPost,
	SemKill,
	SysNo,
}

/// convert int to syscall id
impl From<usize> for SyscallID {
	fn from(value: usize) -> Self {
		match value {
			x if x == SyscallID::Putchar as usize => SyscallID::Putchar,
			x if x == SyscallID::PrintCons as usize => SyscallID::PrintCons,
			x if x == SyscallID::GetEnvID as usize => SyscallID::GetEnvID,
			x if x == SyscallID::Yield as usize => SyscallID::Yield,
			x if x == SyscallID::EnvDestroy as usize => SyscallID::EnvDestroy,
			x if x == SyscallID::SetTlbModEntry as usize => SyscallID::SetTlbModEntry,
			x if x == SyscallID::MemAlloc as usize => SyscallID::MemAlloc,
			x if x == SyscallID::MemMap as usize => SyscallID::MemMap,
			x if x == SyscallID::MemUnmap as usize => SyscallID::MemUnmap,
			x if x == SyscallID::Exofork as usize => SyscallID::Exofork,
			x if x == SyscallID::SetEnvStatus as usize => SyscallID::SetEnvStatus,
			x if x == SyscallID::SetTrapframe as usize => SyscallID::SetTrapframe,
			x if x == SyscallID::Panic as usize => SyscallID::Panic,
			x if x == SyscallID::IpcTrySend as usize => SyscallID::IpcTrySend,
			x if x == SyscallID::IpcRecv as usize => SyscallID::IpcRecv,
			x if x == SyscallID::CGetC as usize => SyscallID::CGetC,
			x if x == SyscallID::WriteDev as usize => SyscallID::WriteDev,
			x if x == SyscallID::ReadDev as usize => SyscallID::ReadDev,
			x if x == SyscallID::ShmGet as usize => SyscallID::ShmGet,
			x if x == SyscallID::ShmAt as usize => SyscallID::ShmAt,
			x if x == SyscallID::ShmDt as usize => SyscallID::ShmDt,
			x if x == SyscallID::ShmCtl as usize => SyscallID::ShmCtl,
			x if x == SyscallID::SemOpen as usize => SyscallID::SemOpen,
			x if x == SyscallID::SemWait as usize => SyscallID::SemWait,
			x if x == SyscallID::SemPost as usize => SyscallID::SemPost,
			x if x == SyscallID::SemKill as usize => SyscallID::SemKill,
			_ => SyscallID::SysNo
		}
	}
}

/// put char to console
fn sys_putchar(c: i32) {
	printcharc(c as u8);
}

/// print string to console
fn sys_print_cons(s_addr: VirtAddr, num: usize) -> i32 {
	if s_addr + num > UTOP || s_addr >= UTOP || s_addr > s_addr + num {
		Error::Inval.into()
	} else {
		for i in 0..num {
			let off = (s_addr + i).as_ptr::<u8>();
			printcharc(unsafe{*off});
		}
		0
	}
}

/// get current env id
fn sys_get_envid() -> EnvID {
	get_cur_env_id().unwrap_or_default()
}

/// yield
fn sys_yield() -> ! {
	env_sched(1)
}
/// destroy env
fn sys_env_destroy(envid: EnvID) -> i32 {
	let ind = try_or_return!(envid2ind(envid, 1));
	println!("[{:x}] destroying {:x}", get_cur_env_id().unwrap_or_default(), envid);
	env_destroy(ind);
	0
}
/// set tlb mod entry of env
fn sys_set_tlb_mod_entry(envid: EnvID, func: usize) -> i32 {
	let mut em = ENV_MANAGER.borrow_mut();
	let ind = try_or_return!(em.envid2ind(envid, 1));
	em.envs[ind].env_user_tlb_mod_entry = func;
	0
}
/// check if virtual address is valid
#[inline]
fn is_illegal_va(va: VirtAddr) -> bool {
	va < UTEMP || va >= UTOP
}
/// check if range of virtual address is valid
#[inline]
fn is_illegal_va_range(va: VirtAddr, len: usize) -> bool {
	if len == 0 {
		false
	} else {
		va + len < va || va < UTEMP || va + len > UTOP
	}
}
/// alloc memory
fn sys_mem_alloc(envid: EnvID, va: VirtAddr, perm: usize) -> i32 {
	if is_illegal_va(va) {
		return Error::Inval.into();
	}
	let ind = try_or_return!(envid2ind(envid, 1));
	let ppn = try_or_return!(frame_alloc());
	let env = &mut ENV_MANAGER.borrow_mut().envs[ind];
	if let Some(pgdir) = env.env_pgdir.borrow_mut() {
		try_or_return!(pgdir.insert(env.env_asid, ppn, va, perm));
	}
	0
}
/// map memory to userspace
fn sys_mem_map(srcid: EnvID, srcva: VirtAddr, dstid: EnvID, dstva: VirtAddr, perm: usize) -> i32 {
	if is_illegal_va(srcva) || is_illegal_va(dstva) {
		return Error::Inval.into();
	}
	let mut em = ENV_MANAGER.borrow_mut();
	let srcind = try_or_return!(em.envid2ind(srcid, 1));
	let dstind = try_or_return!(em.envid2ind(dstid, 1));
	let ppn = if let Some(pgdir) = em.envs[srcind].env_pgdir.borrow_mut() {
		try_or_return!(pgdir.lookup_ppn(srcva))
	} else {
		return Error::Inval.into();
	};

	let dstenv = &mut em.envs[dstind];
	if let Some(pgdir) = dstenv.env_pgdir.borrow_mut() {
		try_or_return!(pgdir.insert(dstenv.env_asid, ppn, dstva, perm))
	}
	0
}
/// umap memory from user space
fn sys_mem_unmap(envid: EnvID, va: VirtAddr) -> i32 {
	if is_illegal_va(va) {
		return Error::Inval.into();
	}
	let ind = try_or_return!(envid2ind(envid, 1));
	
	let env = &mut ENV_MANAGER.borrow_mut().envs[ind];
	if let Some(pgdir) = env.env_pgdir.borrow_mut() {
		pgdir.remove(env.env_asid, va);
	}
	0
}
/// fork
fn sys_exofork() -> i32 {
	let mut em = ENV_MANAGER.borrow_mut();
	let cur_env_ind = em.cur_env_ind.unwrap_or_default();
	let cur_env = &mut em.envs[cur_env_ind];
	let cur_env_pri = cur_env.env_pri;
	let cur_env_id = cur_env.env_id;
	let envid = try_or_return!(em.alloc(cur_env_id));
	let env_ind = envid.envx();
	let env = &mut em.envs[env_ind];
	env.env_pri = cur_env_pri;

	env.load_tf((KSTACKTOP - size_of::<Trapframe>()) as *const Trapframe);
	env.env_tf.regs[2] = 0;
	env.env_status = EnvStatus::NotRunnable;
	envid.0 as i32
}
/// set env status of env
fn sys_set_env_status(envid: EnvID, status: EnvStatus) -> i32 {
	if status != EnvStatus::Runnable && status != EnvStatus::NotRunnable {
		return Error::Inval.into();
	}
	let mut em = ENV_MANAGER.borrow_mut();
	let ind = try_or_return!(em.envid2ind(envid, 1));
	let env = &mut em.envs[ind];
	let prev = env.env_status;
	env.env_status = status;
	if prev == EnvStatus::Runnable {
		em.env_sched_list.remove(ind);
	}
	if status == EnvStatus::Runnable {
		em.env_sched_list.insert_tail(ind);
	}
	0
}
/// set trap frame of env
fn sys_set_trapframe(envid: EnvID, tf: *const Trapframe) -> i32 {
	if is_illegal_va_range(VirtAddr::from_ptr(tf), size_of::<Trapframe>()) {
		return Error::Inval.into();
	}
	let mut em = ENV_MANAGER.borrow_mut();
	let ind = try_or_return!(em.envid2ind(envid, 1));
	let cur_ind = em.cur_env_ind.unwrap_or_default();
	let tf = unsafe {tf.as_ref()}.unwrap();
	if ind == cur_ind {
		let dst = (KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe;
		unsafe {write_volatile(dst, *tf)};
		return tf.regs[2] as i32;
	} else {
		em.envs[ind].env_tf = *tf;
		return 0;
	}
}
/// panic
fn sys_panic(msg: *const i8) {
	let s = unsafe {CStr::from_ptr(msg)};
	let s = s.to_str().unwrap();
	panic!("{}", s);
}
/// ipc receiving message
fn sys_ipc_recv(dstva: VirtAddr) -> i32 {
	if !dstva.is_null() && is_illegal_va(dstva) {
		return Error::Inval.into();
	}
	let mut em = ENV_MANAGER.borrow_mut();
	let cur_ind = em.cur_env_ind.unwrap_or_default();
	let env = &mut em.envs[cur_ind];
	env.env_ipc_receiving = 1;
	env.env_ipc_dstva = dstva;
	env.env_status = EnvStatus::NotRunnable;

	em.env_sched_list.remove(cur_ind);
	let tf = (KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe;
	let tf = unsafe {
		tf.as_mut()
	}.unwrap();
	tf.regs[2] = 0;
	drop(em);
	env_sched(1);
}
/// ipc send message
fn sys_ipc_try_send(envid: EnvID, value: usize, srcva: VirtAddr, perm: usize) -> i32 {
	if !srcva.is_null() && is_illegal_va(srcva) {
		return Error::Inval.into();
	}
	let mut em = ENV_MANAGER.borrow_mut();
	let cur_ind = em.cur_env_ind.unwrap_or_default();
	let cur_env_id = em.envs[cur_ind].env_id;
	let ind = try_or_return!(em.envid2ind(envid, 0));
	let env = &mut em.envs[ind];
	let recving = env.env_ipc_receiving;
	if recving == 0 {
		return Error::IpcNotRecv.into();
	}

	env.env_ipc_value = value;
	env.env_ipc_from = cur_env_id.0;
	env.env_ipc_perm = PTE_V | perm;
	env.env_ipc_receiving = 0;
	env.env_status = EnvStatus::Runnable;
	let dstva = env.env_ipc_dstva;
	let asid = env.env_asid;


	em.env_sched_list.insert_tail(ind);
	if !srcva.is_null() {
		let cur_env = &mut em.envs[cur_ind];
		let ppn = if let Some(pgdir) = cur_env.env_pgdir.borrow_mut() {
			try_or_return!(pgdir.lookup_ppn(srcva))
		} else {
			return Error::Inval.into();
		};
		if let Some(pgdir) = em.envs[ind].env_pgdir.borrow_mut() {
			try_or_return!(pgdir.insert(asid, ppn, dstva, perm));
		}
	}
	0
}
/// get char
fn sys_cgetc() -> i32 {
	scancharc() as i32
}
/// console address
pub const CONSOLE_ADDR: PhysAddr = PhysAddr::new(0x180003f8);
/// disk address
pub const DISK_ADDR: PhysAddr = PhysAddr::new(0x180001f0);
/// console len
pub const CONSOLE_LEN: usize = 0x20;
/// disk len
pub const DISK_LEN: usize = 0x8;
/// write bytes to device
fn sys_write_dev(va: VirtAddr, pa: PhysAddr, len: usize) -> i32 {
	if is_illegal_va_range(va, len) {
		return -(Error::Inval as i32);
	}
	if !(pa >= CONSOLE_ADDR && pa + len <= CONSOLE_ADDR + CONSOLE_LEN) && !(pa >= DISK_ADDR && pa + len <= DISK_ADDR + DISK_LEN) {
		return -(Error::Inval as i32);
	}
	if len != 1 && len != 2 && len != 4 {
		return -(Error::Inval as i32);
	}
	match len {
		1 => DeviceManager.write::<u8>(va, pa),
		2 => DeviceManager.write::<u16>(va, pa),
		4 => DeviceManager.write::<u32>(va, pa),
		_ => {}
	};
	0
}
/// read bytes from device
fn sys_read_dev(va: VirtAddr, pa: PhysAddr, len: usize) -> i32 {
	if is_illegal_va_range(va, len) {
		return -(Error::Inval as i32);
	}
	if !(pa >= CONSOLE_ADDR && pa + len <= CONSOLE_ADDR + CONSOLE_LEN) && !(pa >= DISK_ADDR && pa + len <= DISK_ADDR + DISK_LEN) {
		return -(Error::Inval as i32);
	}
	if len != 1 && len != 2 && len != 4 {
		return -(Error::Inval as i32);
	}
	match len {
		1 => DeviceManager.read::<u8>(va, pa),
		2 => DeviceManager.read::<u16>(va, pa),
		4 => DeviceManager.read::<u32>(va, pa),
		_ => {}
	};
	0
}

/// get or create shared memory
fn sys_shmget(key: usize, size: usize) -> i32 {
	try_or_return!(shm_get(key, size))
}
/// map shared memory at va
fn sys_shmat(id: usize, va: VirtAddr, perm: usize) -> i32 {
	let mut em = ENV_MANAGER.borrow_mut();
	let ind = em.cur_env_ind.unwrap();
	let env = &mut em.envs[ind];
	if let Some(pgdir) = &mut env.env_pgdir {
		try_or_return!(shm_at(id, va, env.env_asid, pgdir, perm))
	}
	0
}
/// deattach sharede memory at va
fn sys_shmdt(id: usize, va: VirtAddr) -> i32 {
	let mut em = ENV_MANAGER.borrow_mut();
	let ind = em.cur_env_ind.unwrap();
	let env = &mut em.envs[ind];
	if let Some(pgdir) = &mut env.env_pgdir {
		try_or_return!(shm_dt(id, va, env.env_asid, pgdir))
	}
	0
}
/// mark shared memory to be removed
fn sys_shmctl(id: usize, ctl: usize) -> i32 {
	if ctl == ShmCtl::Rmid as usize {
		try_or_return!(shm_rmid(id));
	}
	0
}
/// open a semaphore
fn sys_semopen(id: usize, n: isize) -> i32 {
	let mut sem_manager = SEM_MAMANER.borrow_mut();
	sem_manager.sem_open(id, n);
	0
}
/// wait a semaphore
fn sys_semwait(id: usize) -> i32 {
	let mut sem_manager = SEM_MAMANER.borrow_mut();
	let sem = sem_manager.get(id);
	let current = sem.load(core::sync::atomic::Ordering::Relaxed);
	if current == 0 {
		1
	} else {
		let _ = sem.compare_exchange(current, current - 1, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed);
		0
	}
}
/// post a semaphore
fn sys_sempost(id: usize) {
	let mut sem_manager = SEM_MAMANER.borrow_mut();
	let sem = sem_manager.get(id);
	let current = sem.load(core::sync::atomic::Ordering::Relaxed);
	let _ = sem.compare_exchange(current, current + 1, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed);
}
/// kill a semaphore
fn sys_semkill(id: usize) {
	let mut sem_manager = SEM_MAMANER.borrow_mut();
	sem_manager.sem_free(id);
}

/// get syscall func address from syscall id
#[inline]
fn get_syscall(id: SyscallID) -> usize {
	match id {
		SyscallID::Putchar => sys_putchar as usize,
		SyscallID::PrintCons => sys_print_cons as usize,
		SyscallID::GetEnvID => sys_get_envid as usize,
		SyscallID::Yield => sys_yield as usize,
		SyscallID::EnvDestroy => sys_env_destroy as usize,
		SyscallID::SetTlbModEntry => sys_set_tlb_mod_entry as usize,
		SyscallID::MemAlloc => sys_mem_alloc as usize,
		SyscallID::MemMap => sys_mem_map as usize,
		SyscallID::MemUnmap => sys_mem_unmap as usize,
		SyscallID::Exofork => sys_exofork as usize,
		SyscallID::SetEnvStatus => sys_set_env_status as usize,
		SyscallID::SetTrapframe => sys_set_trapframe as usize,
		SyscallID::Panic => sys_panic as usize,
		SyscallID::IpcTrySend => sys_ipc_try_send as usize,
		SyscallID::IpcRecv => sys_ipc_recv as usize,
		SyscallID::CGetC => sys_cgetc as usize,
		SyscallID::WriteDev => sys_write_dev as usize,
		SyscallID::ReadDev => sys_read_dev as usize,
		SyscallID::ShmGet => sys_shmget as usize,
		SyscallID::ShmAt => sys_shmat as usize,
		SyscallID::ShmDt => sys_shmdt as usize,
		SyscallID::ShmCtl => sys_shmctl as usize,
		SyscallID::SemOpen => sys_semopen as usize,
		SyscallID::SemWait => sys_semwait as usize,
		SyscallID::SemPost => sys_sempost as usize,
		SyscallID::SemKill => sys_semkill as usize,
		SyscallID::SysNo => panic!("No such syscall"),
	}
}

/// syscall
pub struct Syscall;

impl Syscall {
	/// do syscall body
	pub fn do_syscall(&self, tf: &mut Trapframe) {
		let sysno = tf.regs[4];
		if sysno >= SyscallID::SysNo as usize {
			tf.regs[2] = -(Error::NoSys as i32) as usize;
			return;
		}
		tf.cp0_epc += 4;
		let func_ptr = get_syscall(SyscallID::from(sysno)) as *const ();
		let arg1 = tf.regs[5];
		let arg2 = tf.regs[6];
		let arg3 = tf.regs[7];
		let sp = tf.regs[29] as *mut usize;
		let sp = unsafe{ slice::from_raw_parts(sp, 6)};
		let arg4 = sp[4];
		let arg5 = sp[5];
		let func: fn(usize, usize, usize, usize, usize) -> i32 = unsafe {
			mem::transmute::<>(func_ptr)
		};
	
		let ret = func(arg1, arg2, arg3, arg4, arg5);
		tf.regs[2] = ret as usize;
	}
}

/// do syscall
#[no_mangle]
pub extern "C" fn do_syscall(tf: &mut Trapframe) {
	Syscall.do_syscall(tf);
}
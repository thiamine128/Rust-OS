use core::{default, ffi::CStr, mem::{self, size_of}, num, ptr::{addr_of, slice_from_raw_parts, write_volatile}, slice, usize};


use crate::{env::{curenv_id, env_accept, env_alloc, env_asid, env_destroy, env_pgdir, env_sched, env_set_tlb_mod_entry, envid2ind, set_env_tf, EnvID}, err::Error, exception::traps::Trapframe, memory::{frame::frame_alloc, mmu::{PhysAddr, VirtAddr, KSEG1, KSTACKTOP, PTE_V, UTEMP, UTOP}}, print::{printcharc, scancharc}, println};

use super::{EnvStatus, ASID, ENV_MANAGER};

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
	SysNo,
}

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
			_ => SyscallID::SysNo
		}
	}
}

fn sys_putchar(c: i32) {
	printcharc(c as u8);
}

fn sys_print_cons(s_addr: VirtAddr, num: usize) -> i32 {
	if s_addr + num > UTOP || s_addr >= UTOP || s_addr > s_addr + num {
		return -(Error::Inval as i32);
	} else {
		for i in 0..num {
			let off = (s_addr + i).as_ptr::<u8>();
			printcharc(unsafe{*off});
		}
		0
	}
}

fn sys_get_envid() -> EnvID {
	curenv_id()
}

fn sys_yield() -> ! {
	env_sched(1)
}

fn sys_env_destroy(envid: EnvID) -> i32 {
	match envid2ind(envid, 1) {
		Ok(ind) => {
			let curenv_id = curenv_id();
			println!("[{:x}] destroying {:x}", curenv_id, envid);
			env_destroy(ind);
			0
		},
		Err(err) => {
			-(err as i32)
		}
	}
}

fn sys_set_tlb_mod_entry(envid: EnvID, func: usize) -> i32 {
	match envid2ind(envid, 1) {
		Ok(ind) => {
			env_set_tlb_mod_entry(ind, func);
			0
		},
		Err(err) => {
			-(err as i32)
		}
	}
}
#[inline]
fn is_illegal_va(va: VirtAddr) -> bool {
	va < UTEMP || va >= UTOP
}

#[inline]
fn is_illegal_va_range(va: VirtAddr, len: usize) -> bool {
	if len == 0 {
		false
	} else {
		va + len < va || va < UTEMP || va + len > UTOP
	}
}

fn sys_mem_alloc(envid: EnvID, va: VirtAddr, perm: usize) -> i32 {
	if is_illegal_va(va) {
		return -(Error::Inval as i32);
	}
	let ind = envid2ind(envid, 1);
	if let Err(err) = ind {
		return -(err as i32);
	}
	let ind = ind.unwrap();
	let ppn = frame_alloc();
	if let Err(err) = ppn {
		return -(err as i32);
	}
	let ppn = ppn.unwrap();
	let asid = env_asid(ind);
	let mut result: Result<(), Error> = Ok(());
	env_pgdir(ind, |pgdir| {
		result = pgdir.insert(asid, ppn, va, perm)
	});
	match result {
		Ok(_) => 0,
		Err(err) => -(err as i32)
	}
}

fn sys_mem_map(srcid: EnvID, srcva: VirtAddr, dstid: EnvID, dstva: VirtAddr, perm: usize) -> i32 {
	if is_illegal_va(srcva) || is_illegal_va(dstva) {
		return -(Error::Inval as i32);
	}
	let srcind = envid2ind(srcid, 1);
	if let Err(err) = srcind {
		return -(err as i32);
	}
	let srcind = srcind.unwrap();
	let dstind = envid2ind(dstid, 1);
	if let Err(err) = dstind {
		return -(err as i32);
	}
	let dstind = dstind.unwrap();
	let mut result = Err(Error::Inval);
	env_pgdir(srcind, |pgdir| {
		result = pgdir.lookup_ppn(srcva)
	});
	if result.is_err() {
		return -(Error::Inval as i32);
	}
	let ppn = result.unwrap();
	let asid = env_asid(dstind);
	let mut result = Err(Error::Inval);
	env_pgdir(dstind, |pgdir| {
		result = pgdir.insert(asid, ppn, dstva, perm)
	});
	if let Err(err) = result {
		return -(err as i32);
	}
	0
}

fn sys_mem_unmap(envid: EnvID, va: VirtAddr) -> i32 {
	if is_illegal_va(va) {
		return -(Error::Inval as i32);
	}
	let ind = envid2ind(envid, 1);
	if let Err(err) = ind {
		return -(err as i32);
	}
	let ind = ind.unwrap();
	let asid = env_asid(ind);
	env_pgdir(ind, |pgdir| {
		pgdir.remove(asid, va);
	});
	0
}

fn sys_exofork() -> i32 {
	let curid = curenv_id();
	let envid = env_alloc(curid);
	if let Err(err) = envid {
		return -(err as i32);
	}
	let envid = envid.unwrap();
	let mut curenv_pri = 0;
	env_accept(curid.envx(), |cur| {
		curenv_pri = cur.env_pri;
	});
	set_env_tf(envid.envx(), (KSTACKTOP - size_of::<Trapframe>()) as *const Trapframe);
	env_accept(envid.envx(), |env| {
		env.env_tf.regs[2] = 0;
		env.env_status = EnvStatus::NotRunnable;
		env.env_pri = curenv_pri;
	});
	envid.0 as i32
}

fn sys_set_env_status(envid: EnvID, status: EnvStatus) -> i32 {
	if status != EnvStatus::Runnable && status != EnvStatus::NotRunnable {
		return -(Error::Inval as i32);
	}
	let ind = envid2ind(envid, 1);
	if let Err(err) = ind {
		return -(err as i32);
	}
	let ind = ind.unwrap();
	let mut prev_status = EnvStatus::Runnable;
	env_accept(ind, |env| {
		prev_status = env.env_status;
	});
	let mut em = ENV_MANAGER.borrow_mut();
	if prev_status == EnvStatus::Runnable {
		em.env_sched_list.remove(ind);
	}
	if status == EnvStatus::Runnable {
		em.env_sched_list.insert_tail(ind);
	}
	em.envs[ind].env_status = status;
	drop(em);
	0
}

fn sys_set_trapframe(envid: EnvID, tf: *const Trapframe) -> i32 {
	if is_illegal_va_range(VirtAddr::from_ptr(tf), size_of::<Trapframe>()) {
		return -(Error::Inval as i32);
	}
	let ind = envid2ind(envid, 1);
	if let Err(err) = ind {
		return -(err as i32);
	}
	let ind = ind.unwrap();
	let curind = curenv_id().envx();
	let tf = unsafe {tf.as_ref()}.unwrap();
	if ind == curind {
		let dst = (KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe;
		unsafe {write_volatile(dst, *tf)};
		return tf.regs[2] as i32;
	} else {
		env_accept(ind, |env| {
			env.env_tf = *tf;
		});
		return 0;
	}
}

fn sys_panic(msg: *const i8) {
	let s = unsafe {CStr::from_ptr(msg)};
	let s = s.to_str().unwrap();
	panic!("{}", s);
}

fn sys_ipc_recv(dstva: VirtAddr) -> i32 {
	if !dstva.is_null() && is_illegal_va(dstva) {
		return -(Error::Inval as i32);
	}
	let curind = curenv_id().envx();
	env_accept(curind, |env| {
		env.env_ipc_receiving = 1;
		env.env_ipc_dstva = dstva;
		env.env_status = EnvStatus::NotRunnable;
	});
	ENV_MANAGER.borrow_mut().env_sched_list.remove(curind);
	let tf = (KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe;
	let tf = unsafe {
		tf.as_mut()
	}.unwrap();
	tf.regs[2] = 0;
	env_sched(1);
}

fn sys_ipc_try_send(envid: EnvID, value: usize, srcva: VirtAddr, perm: usize) -> i32 {
	if !srcva.is_null() && is_illegal_va(srcva) {
		return -(Error::Inval as i32);
	}
	let curind = curenv_id();
	let ind = envid2ind(envid, 0);
	if let Err(err) = ind {
		return -(err as i32);
	}
	let ind = ind.unwrap();
	let mut recving = 0;
	env_accept(ind, |env| {
		recving = env.env_ipc_receiving;
	});
	if recving == 0 {
		return -(Error::IpcNotRecv as i32);
	}
	let mut asid = ASID::zero();
	let mut dstva = VirtAddr::zero();
	env_accept(ind, |env| {
		env.env_ipc_value = value;
		env.env_ipc_from = curind.0;
		asid = env.env_asid;
		env.env_ipc_perm = PTE_V | perm;
		env.env_ipc_receiving = 0;
		env.env_status = EnvStatus::Runnable;
		dstva = env.env_ipc_dstva;
	});
	ENV_MANAGER.borrow_mut().env_sched_list.insert_tail(ind);
	if !srcva.is_null() {
		let mut result = Err(Error::Inval);
		env_pgdir(curind.envx(), |pgdir| {
			result = pgdir.lookup_ppn(srcva);
		});
		if let Err(err) = result {
			return -(Error::Inval as i32);
		}
		let ppn = result.unwrap();
		let mut result = Err(Error::Inval);
		env_pgdir(ind, |pgdir| {
			result = pgdir.insert(asid, ppn, dstva, perm);
		});
	}
	0
}

fn sys_cgetc() -> i32 {
	scancharc() as i32
}

pub const CONSOLE_ADDR: PhysAddr = PhysAddr::new(0x180003f8);
pub const DISK_ADDR: PhysAddr = PhysAddr::new(0x180001f0);
pub const CONSOLE_LEN: usize = 0x20;
pub const DISK_LEN: usize = 0x8;

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
	let kva = VirtAddr::new(pa.as_usize() | KSEG1);
	match len {
		1 => {
			let va = va.as_ptr::<u8>();
			let kva = kva.as_mut_ptr::<u8>();
			 unsafe { *kva = *va }
		},
		2 => {
			let va = va.as_ptr::<u16>();
			let kva = kva.as_mut_ptr::<u16>();
			 unsafe { *kva = *va }
		},
		4 => {
			let va = va.as_ptr::<u32>();
			let kva = kva.as_mut_ptr::<u32>();
			 unsafe { *kva = *va }
		},
		_ => {}
	};
	0
}

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
	let kva = VirtAddr::new(pa.as_usize() | KSEG1);
	match len {
		1 => {
			let va = va.as_mut_ptr::<u8>();
			let kva = kva.as_ptr::<u8>();
			 unsafe { *va = *kva }
		},
		2 => {
			let va = va.as_mut_ptr::<u16>();
			let kva = kva.as_ptr::<u16>();
			 unsafe { *va = *kva }
		},
		4 => {
			let va = va.as_mut_ptr::<u32>();
			let kva = kva.as_ptr::<u32>();
			 unsafe { *va = *kva }
		},
		_ => {}
	};
	0
}

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
		SyscallID::SysNo => panic!("No such syscall"),
	}
}

#[no_mangle]
pub extern "C" fn do_syscall(tf: &mut Trapframe) {
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
pub mod bare;
pub mod schedule;
pub mod syscall;
pub mod sem;

use core::{fmt::{self, LowerHex}, mem::size_of, ptr::{addr_of, addr_of_mut, copy}};

use alloc::vec::Vec;

use crate::{err::Error, exception::traps::{Trapframe, STATUS_EXL, STATUS_IE, STATUS_IM7, STATUS_UM}, memory::{frame::{frame_alloc, frame_base_phy_addr, frame_base_size, frame_decref, frame_incref}, mmu::{PhysAddr, PhysPageNum, VirtAddr, KSTACKTOP, NASID, PDSHIFT, PGSHIFT, PTE_G, PTE_V, UENVS, UPAGES, USTACKTOP, UTOP, UVPT}, page_table::{PageTable, Pte, PAGE_TABLE_ENTRIES}, tlb::tlb_invalidate}, println, sync::cell::UPSafeCell, util::{elf::{elf_from, elf_load_seg, Elf32Phdr, PT_LOAD}, queue::IndexLink}};

const LOG2NENV: usize = 10;
const NENV: usize = 1 << LOG2NENV;


static ENV_MANAGER: UPSafeCell<EnvManager<'static>> = UPSafeCell::new(EnvManager::new());

extern "C" {
    fn env_pop_tf(addr: usize, asid: usize) -> !;
}

#[inline]
pub fn env_init() { ENV_MANAGER.borrow_mut().init(); }

#[inline]
pub fn get_cur_env_ind() -> Option<usize> { ENV_MANAGER.borrow_mut().cur_env_ind }

#[inline]
pub fn get_cur_env_id() -> Option<EnvID>{
    let em = ENV_MANAGER.borrow_mut();
    match em.cur_env_ind {
        Some(ind) => Some(em.envs[ind].env_id),
        None => None
    }
}

#[inline]
pub fn envid2ind(envid: EnvID, checkperm: i32) -> Result<usize, Error> { ENV_MANAGER.borrow_mut().envid2ind(envid, checkperm) }

#[inline]
pub fn user_tlb_mod_entry() -> usize {
    let em = ENV_MANAGER.borrow_mut();
    let ind = em.cur_env_ind.unwrap();
    let ent = em.envs[ind].env_user_tlb_mod_entry;
    drop(em);
    return ent;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ASID(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EnvID(usize);

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnvStatus {
    Free = 0,
    Runnable = 1,
    NotRunnable = 2
}

#[repr(C)]
pub struct Env<'a> {
    env_tf: Trapframe,
    env_id: EnvID,
    env_asid: ASID,
    pub env_parent_id: EnvID,
    env_status: EnvStatus,
    env_pgdir: Option<&'a mut PageTable>,
    env_pri: usize,
    env_ipc_value: usize,
    env_ipc_from: usize,
    env_ipc_receiving: usize,
    env_ipc_dstva: VirtAddr,
    env_ipc_perm: usize,
    env_user_tlb_mod_entry: usize,
    env_runs: usize,
}

pub struct EnvManager<'a> {
    envs: Vec<Env<'a>>,
    base_pgdir: PageTable,
    env_free_list: IndexLink,
    env_sched_list: IndexLink,
    cur_env_ind: Option<usize>,
    asid_bitmap: [usize; NASID / 32],
    alloced_env: usize,
    count: isize
}

impl ASID {
    #[inline]
    pub fn new(v: usize) -> Self {
        Self(v)
    }
    #[inline]
    pub fn zero() -> Self {
        Self(0)
    }
    #[inline]
    pub fn as_usize(self) -> usize {
        self.0
    } 
}

impl EnvID {
    #[inline]
    pub fn new(v: usize) -> Self {
        Self(v)
    }
    #[inline]
    pub fn zero() -> Self {
        Self(0)
    }
    #[inline]
    pub fn as_usize(self) -> usize {
        self.0
    }
    #[inline]
    pub fn envx(self) -> usize {
        self.0 & (NENV - 1)
    }
}

impl Default for EnvID {
    fn default() -> Self {
        Self(0)
    }
}

impl LowerHex for EnvID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt::LowerHex::fmt(&(self.0), f)
    }
}

impl From<usize> for EnvStatus {
    fn from(value: usize) -> Self {
        match value {
			x if x == EnvStatus::Free as usize => EnvStatus::Free,
            x if x == EnvStatus::Runnable as usize => EnvStatus::Runnable,
            _ => EnvStatus::NotRunnable
        }
    }
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env {
            env_tf: Trapframe::new(),
            env_id: EnvID::zero(),
            env_asid: ASID::zero(),
            env_parent_id: EnvID::zero(),
            env_status: EnvStatus::Free,
            env_pgdir: None,
            env_pri: 0,
            env_ipc_value: 0,
            env_ipc_from: 0,
            env_ipc_receiving: 0,
            env_ipc_dstva: VirtAddr::zero(),
            env_ipc_perm: 0,
            env_user_tlb_mod_entry: 0,
            env_runs: 0
        }
    }

    pub fn load_tf(&mut self, tf: *const Trapframe) {
        self.env_tf = unsafe {*tf};
    }
}

impl<'a> EnvManager<'a> {
    #[inline]
    pub const fn new() -> Self {
        EnvManager {
            envs: Vec::new(),
            base_pgdir: PageTable::new(),
            env_free_list: IndexLink::new(),
            env_sched_list: IndexLink::new(),
            cur_env_ind: None,
            asid_bitmap: [0; NASID / 32],
            alloced_env: 0,
            count: 0
        }
    }

    #[inline]
    pub fn init(&mut self) {
        self.envs.resize_with(NENV, || {
            Env::new()
        });
        self.env_free_list.init(NENV);
        self.env_sched_list.init(NENV);
        for i in (0..NENV).rev() {
            self.envs[i].env_status = EnvStatus::Free;
            self.env_free_list.insert_head(i);
        }

        self.base_pgdir.map_segment(ASID::zero(), frame_base_phy_addr(), UPAGES, frame_base_size(), PTE_G);
        self.base_pgdir.map_segment(ASID::zero(), PhysAddr::from_kva(self.envs_raw_ptr()), UENVS, self.envs_size(), PTE_G);
    }

    #[inline]
    fn envs_raw_ptr(&self) -> VirtAddr {
        VirtAddr::from_ptr(self.envs.as_ptr())
    }
    
    #[inline]
    fn envs_size(&self) -> usize {
        self.envs.len() * size_of::<Env>()
    }

    #[inline]
    pub fn asid_alloc(&mut self) -> Result<ASID, Error> {
        for i in 0..NASID {
            let index = i >> 5;
            let inner = i & 31;
            if (self.asid_bitmap[index] & (1 << inner)) == 0 {
                self.asid_bitmap[index] |= 1 << inner;
                return Ok(ASID::new(i));
            }
        }
        Err(Error::NoFreeEnv)
    }

    #[inline]
    pub fn asid_free(&mut self, asid: ASID) {
        let i = asid.as_usize();
        let index = i >> 5;
        let inner = i & 31;
        self.asid_bitmap[index] &= !(1 << inner);
    }

    #[inline]
    pub fn mkenvid(&mut self, ind: usize) -> EnvID {
        self.alloced_env += 1;
        return EnvID::new((self.alloced_env << (1 + LOG2NENV)) | ind);
    }

    #[inline]
    fn setup(&mut self, ind: usize) -> Result<(), Error> {
        let env = &mut self.envs[ind];
        let ppn = frame_alloc()?;
        frame_incref(ppn);
        let pgdir_addr = ppn.into_kva().as_mut_ptr::<PageTable>();
        env.env_pgdir = unsafe {pgdir_addr.as_mut()};

        if let Some(pgdir) = &mut env.env_pgdir {
            for i in UTOP.pdx()..UVPT.pdx() {
                (*pgdir).set_entry(i, self.base_pgdir.get_entry(i));
            }
            pgdir.set_entry(UVPT.pdx(), Pte::new_from_ppn(ppn, PTE_V));
        }
        Ok(())
    }
    #[inline]
    pub fn alloc(&mut self, parent_id: EnvID) -> Result<EnvID, Error>{
        if self.env_free_list.is_empty() {
            return Err(Error::NoFreeEnv);
        }
        let ind = self.env_free_list.first().unwrap();
        self.setup(ind)?;
        let envid = self.mkenvid(ind);
        let asid = self.asid_alloc()?;
        let e = &mut self.envs[ind];
        e.env_user_tlb_mod_entry = 0;
        e.env_runs = 0;
        e.env_id = envid;
        e.env_asid = asid;
        e.env_parent_id = parent_id;
        e.env_tf.cp0_status = STATUS_IM7 | STATUS_IE | STATUS_EXL | STATUS_UM;
        e.env_tf.regs[29] = USTACKTOP.as_usize() - 4 - 4;
        self.env_free_list.remove(ind);
        Ok(envid)
    }
    #[inline]
    pub fn envid2ind(&self, id: EnvID, checkperm: i32) -> Result<usize, Error> {
        
        if id.0 == 0 {
            return Ok(self.cur_env_ind.unwrap());
        }
        
        let cur_env_id = self.envs[self.cur_env_ind.unwrap()].env_id;
        let e = &self.envs[id.envx()];
        if e.env_status == EnvStatus::Free || e.env_id != id {
            return Err(Error::BadEnv)
        }
        if checkperm != 0 {
            if e.env_id != cur_env_id && e.env_parent_id != cur_env_id {
                return Err(Error::BadEnv);
            }
        }
        Ok(id.envx())
    }
    #[inline]
    pub fn get_env(&mut self, ind: usize) -> &mut Env<'a>{
        &mut self.envs[ind]
    }
    #[inline]
    pub fn free(&mut self, ind: usize) {
        println!("[{:x}] free env [{:x}]", match self.cur_env_ind {
            Some(ind) => self.get_env(ind).env_id.0,
            None => EnvID::zero().0
        }, self.get_env(ind).env_id.0);

        let env = &mut self.envs[ind];
        if let Some(pgdir) = &mut env.env_pgdir {
            for pdeno in 0..UTOP.pdx() {
                if !pgdir.get_entry(pdeno).valid() {
                    continue;
                }
                let pte = pgdir.get_entry(pdeno);
                let addr = pte.addr().into_kva().as_mut_ptr::<PageTable>();
                let pgtable: &mut PageTable = unsafe { addr.as_mut() }.unwrap();
                for pteno in 0..PAGE_TABLE_ENTRIES {
                    if pgtable.get_entry(pteno).valid() {
                        pgdir.remove(env.env_asid, VirtAddr::new((pdeno << PDSHIFT) | (pteno << PGSHIFT)));
                    }
                }
                pgdir.set_entry(pdeno, Pte::new(0));
                frame_decref(pte.ppn());
                tlb_invalidate(env.env_asid, UVPT + (pdeno << PGSHIFT));
            }
            frame_decref(PhysPageNum::from(PhysAddr::from_kva(VirtAddr::from_ptr(addr_of_mut!(**pgdir)))));
        }
        let asid = env.env_asid;
        env.env_status = EnvStatus::Free;
        self.asid_free(asid);
        tlb_invalidate(asid, UVPT + (UVPT.pdx() << PGSHIFT));
        self.env_free_list.insert_head(ind);
        self.env_sched_list.remove(ind);

    }
    #[inline]
    pub fn create(&mut self, binary: &[u8], size: usize, priority: usize) -> EnvID {
        let envid = self.alloc(EnvID::zero()).unwrap();
        let ind = envid.envx();
        let env: &mut Env<'a> = &mut self.envs[ind];
        env.env_pri = priority;
        env.env_status = EnvStatus::Runnable;
        load_icode(env, binary, size);
        self.env_sched_list.insert_head(ind);
        envid
    }
}

#[inline]
pub fn env_destroy(ind: usize) {
    let mut em = ENV_MANAGER.borrow_mut();
    em.free(ind);
    let cur_env_ind = em.cur_env_ind.unwrap();
    if ind == cur_env_ind {
        em.cur_env_ind = None;
        println!("I am killed ...");
        drop(em);
        env_sched(1);
    }
}

pub fn pre_env_run(_: usize) {

}

pub fn env_run(ind: usize) -> ! {
    //pre_env_run(ind);
    let mut em = ENV_MANAGER.borrow_mut();
    let env = em.get_env(ind);
    assert!(env.env_status == EnvStatus::Runnable);
    
    let cur = em.cur_env_ind;
    if let Some(cur_ind) = cur {
        let kstacktop = (KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe;
        em.get_env(cur_ind).load_tf(kstacktop);
        
    }

    em.cur_env_ind = Some(ind);
    let curenv = em.get_env(ind);
    curenv.env_runs += 1;
    
    //println!("{:x} from {:x}", curenv.env_id.0, curenv.env_tf.cp0_epc);
    let tf_addr = addr_of!(curenv.env_tf) as usize;
    let asid = curenv.env_asid.as_usize();
    drop(em);
    unsafe {env_pop_tf(tf_addr, asid)}

}
pub fn env_sched(y: i32) -> ! {
    let mut em = ENV_MANAGER.borrow_mut();
    em.count -= 1;
    let e = em.cur_env_ind;
    let next_run;
    if y != 0 || em.count == 0 || e.is_none() || em.get_env(e.unwrap()).env_status != EnvStatus::Runnable {
        if em.env_sched_list.is_empty() {
            panic!("Sched list empty");
        }
        if e.is_some() && em.get_env(e.unwrap()).env_status == EnvStatus::Runnable {
            em.env_sched_list.remove(e.unwrap());
            em.env_sched_list.insert_tail(e.unwrap());
        }
        next_run = em.env_sched_list.first().unwrap();
        em.count = em.get_env(next_run).env_pri as isize;
    } else {
        next_run = e.unwrap();
    }
    drop(em);
    env_run(next_run);
}

fn load_icode_mapper(env: &mut Env, va: VirtAddr, offset: usize, perm: usize, src: Option<&[u8]>, len: usize) -> Result<(), Error> {
    let ppn = frame_alloc()?;
    if src.is_some() {
        let dst = (ppn.into_kva() + offset).as_mut_ptr::<u8>();
        let src_addr = src.unwrap().as_ptr();
        unsafe { copy(src_addr, dst, len); }
    }
    let asid = env.env_asid;
    if let Some(pgdir) = &mut env.env_pgdir {
        pgdir.insert(asid, ppn, va, perm)?;
    }
    Ok(())
}

fn load_icode(env: &mut Env, binary: &[u8], size: usize) {
    let ehdr = elf_from(binary, size);
    if ehdr.is_none() {
        panic!("bad elf at {:p}", binary.as_ptr());
    }
    let ehdr = ehdr.unwrap();
    for phdr_off in ehdr.phdr_iter() {
        let phdr_addr = (binary.as_ptr() as usize + phdr_off) as *const Elf32Phdr;
        let phdr = unsafe { phdr_addr.as_ref() }.unwrap();
        if phdr.p_type == PT_LOAD {
            elf_load_seg(phdr, &binary[phdr.p_offset as usize..], |va, offset, perm, bin, size| {
                load_icode_mapper(env, va, offset, perm, bin, size)
            }).unwrap();
        }
    }
    env.env_tf.cp0_epc = ehdr.e_entry as usize;
}

pub fn insert_sched(envid: EnvID) {
    let sched_list = &mut ENV_MANAGER.borrow_mut().env_sched_list;
    sched_list.insert_tail(envid.envx());
}
pub fn env_alloc(parent_id: EnvID) -> Result<EnvID, Error> {
    ENV_MANAGER.borrow_mut().alloc(parent_id)
}
pub fn env_free(ind: usize) {
    ENV_MANAGER.borrow_mut().free(ind);
}
pub fn env_create(binary: &[u8], size: usize, priority: usize) -> EnvID {
    ENV_MANAGER.borrow_mut().create(binary, size, priority)
}
pub fn cur_pgdir<F>(mut f: F)
where
    F : FnMut(&mut PageTable) {
    let ind = ENV_MANAGER.borrow_mut().cur_env_ind.unwrap();
    if let Some(pgdir) = &mut ENV_MANAGER.borrow_mut().get_env(ind).env_pgdir {
        f(pgdir);
    }
}
pub fn env_pgdir<F>(ind: usize, mut f: F)
where
    F : FnMut(&mut PageTable) {
    if let Some(pgdir) = &mut ENV_MANAGER.borrow_mut().get_env(ind).env_pgdir {
        f(pgdir);
    }
}

#[macro_export]
macro_rules! env_create_pri {
    ($name: ident, $pri: expr) => {
        crate::env::env_create(&concat_idents!(binary_, $name, _start), concat_idents!(binary_, $name, _size), $pri)
    };
}
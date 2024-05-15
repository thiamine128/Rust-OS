pub mod bare;
pub mod schedule;

use core::{borrow::Borrow, mem::size_of, ptr::{addr_of, addr_of_mut}};

use alloc::vec::Vec;
use spin::Mutex;

use crate::{env::test::pre_env_run, err::Error, exception::traps::{Trapframe, STATUS_EXL, STATUS_IE, STATUS_IM7, STATUS_UM}, memory::{frame::{frame_alloc, frame_base_phy_addr, frame_base_size, frame_decref, frame_incref}, mmu::{PhysAddr, PhysPageNum, VirtAddr, KSTACKTOP, NASID, PDSHIFT, PGSHIFT, PTE_G, PTE_V, UENVS, UPAGES, USTACKTOP, UTOP, UVPT}, page_table::{PageTable, Pte, PAGE_TABLE_ENTRIES}, tlb::tlb_invalidate}, println, util::{elf::{elf_from, elf_load_seg, Elf32Phdr, PT_LOAD}, queue::IndexLink}};

const LOG2NENV: usize = 10;
const NENV: usize = 1 << LOG2NENV;


static ENV_MANAGER: Mutex<EnvManager<'static>> = Mutex::new(EnvManager::new());

extern "C" {
    fn env_pop_tf(addr: usize, asid: usize);
}

#[inline]
pub fn env_init() {
    ENV_MANAGER.lock().init();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ASID(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EnvID(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnvStatus {
    Free = 0,
    Runnable = 1,
    NotRunnable = 2
}

pub struct Env<'a> {
    env_tf: Trapframe,
    env_id: EnvID,
    env_asid: ASID,
    env_parent_id: EnvID,
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
    pub fn envid2env(&mut self, id: EnvID, checkperm: i32) -> Result<&mut Env<'a>, Error> {
        if self.cur_env_ind.is_none() {
            panic!("No env is running.");
        } else {
            if id.0 == 0 {
                return Ok(&mut self.envs[self.cur_env_ind.unwrap()]);
            }
        }
        
        let cur_env_id = self.envs[self.cur_env_ind.unwrap()].env_id;
        let e = &mut self.envs[id.envx()];
        if e.env_status == EnvStatus::Free || e.env_id != id {
            return Err(Error::BadEnv)
        }
        if checkperm != 0 {
            if e.env_id != cur_env_id && e.env_parent_id != cur_env_id {
                return Err(Error::BadEnv);
            }
        }
        Ok(e)
    }
    #[inline]
    pub fn get_env(&mut self, ind: usize) -> &mut Env<'a>{
        &mut self.envs[ind]
    }
    #[inline]
    pub fn free(&mut self, ind: usize) {
        println!("[{}] free env [{}]", match self.cur_env_ind {
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
                let pgtable: &mut PageTable = unsafe {
                    addr.as_mut()
                }.unwrap();
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
    #[inline]
    pub fn destroy(&mut self, ind: usize) {
        self.free(ind);
        let cur_env_ind = self.cur_env_ind.unwrap();
        if ind == cur_env_ind {
            self.cur_env_ind = None;
            println!("I am killed ...");
            self.sched(1);
        }
    }

    pub fn prepare_run(&mut self, ind: usize) -> (usize, usize) {
        let p = unsafe {pre_env_run(self, ind)};
        if p.is_some() {
            return p.unwrap();
        }
        let env = self.get_env(ind);
        assert!(env.env_status == EnvStatus::Runnable);
        
        let cur = self.cur_env_ind;
        if let Some(cur_ind) = cur {
            let kstacktop = (KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe;
            self.get_env(cur_ind).env_tf = unsafe {
                *kstacktop
            };
            
        }

        self.cur_env_ind = Some(ind);
        let curenv = self.get_env(ind);
        curenv.env_runs += 1;
        
        //println!("{:x} from {:x}", curenv.env_id.0, curenv.env_tf.cp0_epc);
        let tf_addr = addr_of!(curenv.env_tf) as usize;
        let asid = curenv.env_asid.as_usize();
        (tf_addr, asid)

    }
    pub fn sched(&mut self, y: i32) -> (usize, usize) {
        self.count -= 1;
        let e = self.cur_env_ind;
        let next_run;
        if y != 0 || self.count == 0 || e.is_none() || self.get_env(e.unwrap()).env_status != EnvStatus::Runnable {
            if self.env_sched_list.is_empty() {
                panic!("Sched list empty");
            }
            if e.is_some() && self.get_env(e.unwrap()).env_status == EnvStatus::Runnable {
                self.env_sched_list.remove(e.unwrap());
                self.env_sched_list.insert_tail(e.unwrap());
            }
            next_run = self.env_sched_list.first().unwrap();
            self.count = self.get_env(next_run).env_pri as isize;
        } else {
            next_run = e.unwrap();
        }
        self.prepare_run(next_run)
    }
}

fn load_icode_mapper(env: &mut Env, va: VirtAddr, offset: usize, perm: usize, src: Option<&[u8]>, len: usize) -> Result<(), Error> {
    let ppn = frame_alloc()?;
    let mut hash = 0;
    if src.is_some() {
        let st = ppn.into_kva() + offset;
        for i in 0..len {
            let write_addr = (st + i).as_mut_ptr::<u8>();
            let read = src.unwrap()[i];
            unsafe {*write_addr = read};
            hash = hash + read as i32;
        }
    }
    //println!("map {:p} to {:x} with {}", va, ppn.as_usize() << PGSHIFT, hash);
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
        let phdr = unsafe {
            phdr_addr.as_ref()
        }.unwrap();
        if phdr.p_type == PT_LOAD {
            elf_load_seg(phdr, &binary[phdr.p_offset as usize..], |va, offset, perm, bin, size| {
                load_icode_mapper(env, va, offset, perm, bin, size)
            }).unwrap();
        }
    }
    env.env_tf.cp0_epc = ehdr.e_entry as usize;
}

pub fn insert_sched(envid: EnvID) {
    let sched_list = &mut ENV_MANAGER.lock().env_sched_list;
    sched_list.insert_tail(envid.envx());
}
pub fn env_alloc(parent_id: EnvID) -> Result<EnvID, Error> {
    ENV_MANAGER.lock().alloc(parent_id)
}
pub fn env_free(ind: usize) {
    ENV_MANAGER.lock().free(ind);
}
pub fn env_create(binary: &[u8], size: usize, priority: usize) -> EnvID {
    ENV_MANAGER.lock().create(binary, size, priority)
}
pub fn cur_pgdir<F>(mut f: F)
where
    F : FnMut(&mut PageTable) {
    let ind = ENV_MANAGER.lock().cur_env_ind.unwrap();
    if let Some(pgdir) = &mut ENV_MANAGER.lock().get_env(ind).env_pgdir {
        f(pgdir);
    }
}
#[macro_export]
macro_rules! env_create_pri {
    ($name: ident, $pri: expr) => {
        crate::env::env_create(&concat_idents!(binary_, $name, _start), concat_idents!(binary_, $name, _size), $pri)
    };
}

pub mod test {
    use core::mem::size_of;

    use crate::{exception::traps::Trapframe, memory::mmu::KSTACKTOP, println};

    use super::EnvManager;
    
    pub fn pre_env_run(em: &mut EnvManager, e: usize) -> Option<(usize, usize)>{
        let tfp;
        if Some(e) == em.cur_env_ind {
            let addr = ((KSTACKTOP - size_of::<Trapframe>()) as *mut Trapframe);
            tfp = unsafe {addr.as_ref()}.unwrap();
        } else {
            tfp = &em.get_env(e).env_tf;
        }
        

        let epc = tfp.cp0_epc;
        let v0 = tfp.regs[2];
        if tfp.cp0_epc == 0x400180 {
            println!("env {:x} reached end pc: {:x}, $v0={:x}", em.get_env(e).env_id.as_usize(), epc, v0);
            em.destroy(e);
            Some(em.sched(0))
        } else {
            None
        }
    }
}
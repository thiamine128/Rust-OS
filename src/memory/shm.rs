use crate::{env::ASID, err::Error, println, sync::cell::UPSafeCell};
use super::{frame::{frame_alloc, frame_decref, frame_incref, num_free_frames}, mmu::{PhysPageNum, VirtAddr, PAGE_SIZE}, page_table::PageTable};

pub const SHMALL: usize = 4096;
pub const SHMMNI: usize = 128;

pub static SHM_MANAGER: UPSafeCell<ShmManager> = UPSafeCell::new(ShmManager::new());

pub fn init() {
    let mut shm_manager = SHM_MANAGER.borrow_mut();
    shm_manager.init();
}

pub fn shm_get(key: usize, size: usize) -> Result<i32, Error> {
    let nframes = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    if nframes == 0 {
        return Err(Error::Inval);
    }
    let mut shm_manager = SHM_MANAGER.borrow_mut();
    let mut shm_id = None;
    for i in 0..shm_manager.shms.len() {
        if shm_manager.shms[i].key == key && shm_manager.shms[i].nblocks > 0 {
            shm_id = Some(i);
        }
    }
    if shm_id.is_none() || key == 0 {
        shm_id = Some(shm_manager.alloc(nframes)?);
        shm_manager.shms[shm_id.unwrap()].key = key;
    }
    Ok(shm_id.unwrap() as i32)
}

pub fn shm_at(id: usize, va: VirtAddr, asid: ASID, pgdir: &mut PageTable, perm: usize) -> Result<(), Error>{
    let mut shm_manager = SHM_MANAGER.borrow_mut();
    if id >= SHMMNI || shm_manager.shms[id].nblocks == 0 {
        return Err(Error::Inval);
    }
    shm_manager.map(id, va, asid, pgdir, perm)?;
    shm_manager.shms[id].shm_ref += 1;
    Ok(())
}

pub fn shm_dt(id: usize, va: VirtAddr, asid: ASID, pgdir: &mut PageTable) -> Result<(), Error> {
    let mut shm_manager = SHM_MANAGER.borrow_mut();
    if id >= SHMMNI || shm_manager.shms[id].nblocks == 0 {
        return Err(Error::Inval);
    }

    let mut va = va;
    for _ in 0..shm_manager.shms[id].nblocks {
        pgdir.remove(asid, va);
        va += PAGE_SIZE;
    }
    shm_manager.deattach(id);
    Ok(())
}

pub fn shm_rmid(id: usize) -> Result<(), Error> {
    let mut shm_manager = SHM_MANAGER.borrow_mut();
    if shm_manager.shms[id].nblocks == 0 {
        return Err(Error::Inval);
    }
    shm_manager.rmid(id);
    Ok(())
}

#[repr(usize)]
pub enum ShmCtl {
    Rmid = 1
}

#[derive(Clone, Copy)]
pub struct ShmBlock {
    ppn: PhysPageNum,
    next: Option<usize>
}

#[derive(Clone, Copy)]
pub struct Shm {
    head: usize,
    nblocks: usize,
    shm_ref: usize,
    rmid: u8,
    key: usize,
}

pub struct ShmManager {
    blocks: [ShmBlock; SHMALL],
    free_blocks: [usize; SHMALL],
    blockstop: usize,
    shms: [Shm; SHMMNI],
    free_ids: [usize; SHMMNI],
    idstop: usize,
}

impl ShmBlock {
    #[inline]
    pub const fn new() -> Self {
        Self {
            ppn: PhysPageNum::new(0),
            next: None
        }
    }
}

impl Shm {
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: 0,
            nblocks: 0,
            shm_ref: 0,
            rmid: 0,
            key: 0
        }
    }
}

impl ShmManager {
    #[inline]
    pub const fn new() -> Self {
        Self {
            blocks: [ShmBlock::new(); SHMALL],
            free_blocks: [0; SHMALL],
            shms: [Shm::new(); SHMMNI],
            blockstop: 0,
            free_ids: [0; SHMMNI],
            idstop: 0
        }
    }

    pub fn init(&mut self) {
        for i in 0..SHMALL {
            self.free_blocks[i] = i;
        }
        for i in 0..SHMMNI {
            self.free_ids[i] = i;
        }
        self.blockstop = SHMALL;
        self.idstop = SHMMNI;
    }

    #[inline]
    pub fn alloc(&mut self, nframes: usize) -> Result<usize, Error> {
        if nframes > self.blockstop || self.idstop == 0 {
            return Err(Error::NoSpc);
        }
        if nframes > num_free_frames() {
            return Err(Error::NoMem);
        }
        let id = self.free_ids[self.idstop - 1];
        self.idstop -= 1;
        let mut blk = self.free_blocks[self.blockstop - 1];
        self.blockstop -= 1;
        self.shms[id].nblocks = nframes;
        self.shms[id].head = blk;
        for i in 0..nframes {
            let ppn = frame_alloc()?;
            frame_incref(ppn);
            self.blocks[blk].ppn = ppn;
            if i + 1 != nframes {
                let next = self.free_blocks[self.blockstop - 1];
                self.blockstop -= 1;
                self.blocks[blk].next = Some(next);
                blk = next;
            } else {
                self.blocks[blk].next = None;
            }
        }
        Ok(id)
    }

    #[inline]
    pub fn attach(&mut self, id: usize) {
        self.shms[id].shm_ref += 1;
        let mut blk = self.shms[id].head;
        for _ in 0..self.shms[id].nblocks {
            blk = self.blocks[blk].next.unwrap_or_default();
        }
    }

    #[inline]
    pub fn rmid(&mut self, id: usize) {
        self.shms[id].rmid = 1;
    }

    #[inline]
    pub fn deattach(&mut self, id: usize) {
        self.shms[id].shm_ref -= 1;
        if self.shms[id].shm_ref == 0 && self.shms[id].rmid != 0 {
            self.dealloc(id);
        }
    }

    #[inline]
    pub fn dealloc(&mut self, id: usize) {
        println!("free shm[{}]", id);
        let mut blk = self.shms[id].head;
        for _ in 0..self.shms[id].nblocks {
            frame_decref(self.blocks[blk].ppn);
            blk = self.blocks[blk].next.unwrap_or_default();
        }
        self.shms[id].nblocks = 0;
    }

    #[inline]
    pub fn map(&mut self, id: usize, va: VirtAddr, asid: ASID, pgdir: &mut PageTable, perm: usize) -> Result<(), Error> {
        let mut blk = self.shms[id].head;
        let mut va = va;
        for _ in 0..self.shms[id].nblocks {
            pgdir.insert(asid, self.blocks[blk].ppn, va, perm)?;
            blk = self.blocks[blk].next.unwrap_or_default();
            va += PAGE_SIZE;
        }
        Ok(())
    }
}
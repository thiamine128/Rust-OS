use crate::println;

extern "C" {
    fn handle_int();
    fn handle_tlb();
    fn handle_sys();
    fn handle_mod();
    fn handle_reserved();
}

pub const STATUS_CU3: usize = 0x80000000;
pub const STATUS_CU2: usize = 0x40000000;
pub const STATUS_CU1: usize = 0x20000000;
pub const STATUS_CU0: usize = 0x10000000;
pub const STATUS_BEV: usize = 0x00400000;
pub const STATUS_IM0: usize = 0x0100;
pub const STATUS_IM1: usize = 0x0200;
pub const STATUS_IM2: usize = 0x0400;
pub const STATUS_IM3: usize = 0x0800;
pub const STATUS_IM4: usize = 0x1000;
pub const STATUS_IM5: usize = 0x2000;
pub const STATUS_IM6: usize = 0x4000;
pub const STATUS_IM7: usize = 0x8000;
pub const STATUS_UM: usize = 0x0010;
pub const STATUS_R0: usize = 0x0008;
pub const STATUS_ERL: usize = 0x0004;
pub const STATUS_EXL: usize = 0x0002;
pub const STATUS_IE: usize = 0x0001;

#[no_mangle]
static exception_handlers: [unsafe extern "C" fn(); 32] = {
    let mut template = [handle_reserved as unsafe extern "C" fn();32];
    template[0] = handle_int;
    template[2] = handle_tlb;
    template[3] = handle_tlb;
    //template[1] = handle_mod;
    //template[8] = handle_sys;
    template
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Trapframe {
    pub regs: [usize; 32],
    pub cp0_status: usize,
    pub hi: usize,
    pub lo: usize,
    pub cp0_badvaddr: usize,
    pub cp0_cause: usize,
    pub cp0_epc: usize,
}

impl Trapframe {
    pub fn new() -> Self {
        Trapframe {
            regs: [0; 32],
            cp0_status: 0,
            hi: 0,
            lo: 0,
            cp0_cause: 0,
            cp0_badvaddr: 0,
            cp0_epc: 0
        }
    }
}

#[no_mangle]
pub extern "C" fn do_reserved() {
    println!("do reserved");
}

#[no_mangle]
pub extern "C" fn do_tlb_mod() {
    println!("do tlbmod");
}

#[no_mangle]
pub extern "C" fn do_syscall() {
    println!("do syscall");
}

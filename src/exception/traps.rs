extern "C" {
    fn handle_int();
    fn handle_tlb();
    fn handle_sys();
    fn handle_mod();
    fn handle_reserved();
}

#[repr(C)]
pub struct Trapframe {
    regs: [usize; 32],
    cp0_status: usize,
    hi: usize,
    lo: usize,
    cp0_badvaddr: usize,
    cp0_cause: usize,
    cp0_epc: usize,
}
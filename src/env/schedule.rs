use super::ENV_MANAGER;

extern "C" {
    fn env_pop_tf(addr: usize, asid: usize);
}

#[no_mangle]
pub extern "C" fn schedule(y: i32) {
    let (addr, asid) = ENV_MANAGER.lock().sched(y);
    unsafe {env_pop_tf(addr, asid)};
}
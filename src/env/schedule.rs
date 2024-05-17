use super::env_sched;

#[no_mangle]
pub extern "C" fn schedule(y: i32) {
    env_sched(y);
}
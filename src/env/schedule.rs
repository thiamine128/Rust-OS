use super::{env_sched, ENV_MANAGER};

#[no_mangle]
pub extern "C" fn schedule(y: i32) {
    env_sched(0);
}
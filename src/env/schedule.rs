use super::env_sched;

/// make env sched
#[no_mangle]
pub extern "C" fn schedule(y: i32) {
    env_sched(y);
}
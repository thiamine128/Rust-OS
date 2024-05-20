pub mod queue;
pub mod linked_list;
pub mod bitops;
pub mod elf;

#[macro_export]
macro_rules! try_or_return {
    ($expr:expr $(,)?) => {
        match $expr {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(err) => {
                return err.into();
            }
        }
    };
}
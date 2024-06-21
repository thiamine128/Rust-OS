/// index linked list
pub mod queue;
/// pointer linked list
pub mod linked_list;
/// bit operations
pub mod bitops;
/// elf utils
pub mod elf;

/// try macro in rust
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
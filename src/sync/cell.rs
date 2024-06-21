use core::cell::{RefCell, RefMut};

/// safe for single-core
pub struct UPSafeCell<T> {
    inner: RefCell<T>
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// create a new safe
    pub const fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value)
        }
    }
    /// borrow a ref mut
    #[inline]
    pub fn borrow_mut(&self) -> RefMut<'_, T>{
        self.inner.borrow_mut()
    }
}
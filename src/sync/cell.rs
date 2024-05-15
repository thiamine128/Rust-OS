use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    inner: RefCell<T>
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value)
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T>{
        self.inner.borrow_mut()
    }
}
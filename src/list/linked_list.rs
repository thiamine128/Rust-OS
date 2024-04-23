use core::ptr::{addr_of_mut, null, null_mut, NonNull};

pub struct Link<T> {
    pub next: *mut T,
    pub le_prev: *mut *mut T,
}

pub struct Head<T> {
    pub first: *mut T
}

impl<T> Link<T> {
    pub fn new() -> Self {
        Self {
            next: null_mut(),
            le_prev: null_mut()
        }
    }
}

impl<T> Head<T> {
    pub fn new() -> Self {
        Self {
            first: null_mut()
        }
    }
}

#[macro_export]
macro_rules! list_insert_head {
    ($head:expr, $elm:expr, $link:ident) => {
        $elm.$link.le_prev = core::ptr::addr_of_mut!($head.first);
        $elm.$link.next = $head.first;
        $head.first = $elm;
    };
}

#[macro_export]
macro_rules! list_empty {
    ($head:expr) => {
        $head.first.is_null()
    };
}

#[macro_export]
macro_rules! list_first {
    ($head:expr) => {
        $head.first
    };
}

#[macro_export]
macro_rules! list_remove {
    ($elem:expr, $link:ident) => {
        unsafe {
            if !$elem.$link.next.is_null() {
                $elem.$link.next.as_mut().unwrap().$link.le_prev = $elem.$link.le_prev;
            }
            *($elem.$link.le_prev) = $elem.$link.next
        }
    };
}

#[macro_export]
macro_rules! list_insert_after {
    ($listelm:expr, $elm:expr, $link:ident) => {
        $elm.$link.next = $listelm.$link.next;
        if !$listelm.$link.next.is_null() {
            unsafe {
                (*$listelm.$link.next).$link.le_prev = addr_of_mut!($elm.$link.next);
            }
        }
        $listelm.$link.next = addr_of_mut!($elm);
        $elm.$link.le_prev = addr_of_mut!($listelm.$link.next);
    };
}
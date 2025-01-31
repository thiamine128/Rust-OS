/// mos error codes
#[derive(Debug)]
#[repr(i32)]
pub enum Error {
    Unspecified = 1,
    BadEnv = 2,
    Inval = 3,
    NoMem = 4,
    NoSys = 5,
    NoFreeEnv = 6,
    IpcNotRecv = 7,
    NoDisk = 8,
    MaxOpen = 9,
    NotFound = 10,
    BadPath = 11,
    FileExists = 12,
    NotExec = 13,
    NotMapped = 14,
    NoSpc = 15,
}

impl Into<i32> for Error {
    fn into(self) -> i32 {
        -(self as i32)
    }
}
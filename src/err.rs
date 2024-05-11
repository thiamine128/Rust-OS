#[derive(Debug)]
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
    NotMapped = 14
}
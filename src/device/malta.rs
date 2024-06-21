/// register constant
pub const MALTA_PCIIO_BASE: usize = 0x18000000;
/// register constant
pub const MALTA_SERIAL_BASE: usize = MALTA_PCIIO_BASE + 0x3f8;
/// register constant
pub const MALTA_SERIAL_DATA: usize = MALTA_SERIAL_BASE + 0x0;
/// register constant
pub const MALTA_SERIAL_LSR: usize = MALTA_SERIAL_BASE + 0x5;
/// register constant
pub const MALTA_SERIAL_THR_EMPTY: u8 = 0x20;
/// register constant
pub const MALTA_SERIAL_DATA_READY: u8 = 0x1;
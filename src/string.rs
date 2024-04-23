pub fn memset(mut dst: *mut u8, c: i32, n: usize) -> *mut u8{
    unsafe {
        let dstaddr = dst;
        let max = dst.add(n);
        let byte= (c & 0xff) as u8;
        let word = (byte as u32)  | (byte as u32) << 8 | (byte as u32) << 16 | (byte as u32) << 24;
        while ((dst as u32) & 3) != 0 && dst < max {
            *dst = byte;
            dst = dst.offset(1);
        }

        while dst.offset(4) <= max {
            *(dst as *mut u32) = word;
            dst = dst.offset(4);
        }

        while dst < max {
            *dst = byte;
        }

        dstaddr
    }
}
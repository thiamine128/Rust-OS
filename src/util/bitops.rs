pub const BITS_PER_LONG: usize = 32;
pub const BITS_PER_LONG_LONG: usize = 64;

/*
 * Create a contiguous bitmask starting at bit position @l and ending at
 * position @h. For example
 * GENMASK_ULL(39, 21) gives us the 64bit vector 0x000000ffffe00000.
 */
#[inline]
pub fn genmask(h: usize, l: usize) -> usize {
    ((!0usize) << l) & (!0usize >> (BITS_PER_LONG - 1 - h))
}

#[inline]
pub fn genmask_u64(h: usize, l: usize) -> u64 {
    ((!0u64) << l) & (!0u64 >> (BITS_PER_LONG_LONG - 1 - h))
}

#[inline]
pub fn log_1(n: usize) -> usize {
    if n >= 2 {
        1
    } else {
        0
    }
}

#[inline]
pub fn log_2(n: usize) -> usize {
    if n >= 1 << 2 {
        2 + log_1(n >> 2)
    } else {
        log_1(n)
    }
}

#[inline]
pub fn log_4(n: usize) -> usize {
    if n >= 1 << 4 {
        4 + log_2(n >> 4)
    } else {
        log_2(n)
    }
}

#[inline]
pub fn log_8(n: usize) -> usize {
    if n >= 1 << 8 {
        8 + log_4(n >> 8)
    } else {
        log_4(n)
    }
}

#[inline]
pub fn log2(n: usize) -> usize {
    if n >= 1 << 16 {
        16 + log_8(n >> 16)
    } else {
        log_8(n)
    }
}
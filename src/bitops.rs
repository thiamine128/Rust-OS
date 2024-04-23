pub const BITS_PER_LONG: usize = 32;

#[inline]
pub fn genmask(h: usize, l: usize) -> usize{
    ((!0usize) << l) & (!0usize >> (BITS_PER_LONG - 1 - h))
}
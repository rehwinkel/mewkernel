pub trait SetBits {
    unsafe fn set_bits(self, offset: usize, len: usize, value: usize);
}

impl SetBits for *mut usize {
    unsafe fn set_bits(self, offset: usize, len: usize, value: usize) {
        let mask = ((1 << len) - 1) << offset;
        let value = (core::ptr::read_volatile(self) & !mask) | ((value << offset) & mask);
        core::ptr::write_volatile(self, value);
    }
}

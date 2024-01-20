const SYST_CSR: *mut usize = 0xE000E010 as *mut usize; // Control and status
const SYST_RVR: *mut usize = 0xE000E014 as *mut usize; // Reload value
const SYST_CVR: *mut usize = 0xE000E018 as *mut usize; // Current value

pub unsafe fn enable() {
    SYST_RVR.write_volatile(8000 - 1);
    SYST_CVR.write_volatile(0);
    SYST_CSR.write_volatile(0b111);
}

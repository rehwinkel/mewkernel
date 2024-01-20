#![no_std]
#![no_main]

extern crate mewkernel_rt;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub fn usart2_interrupt_handler() {}

#[no_mangle]
pub fn sv_call_handler() {}

#[no_mangle]
pub fn sys_tick_handler() {}

#[no_mangle]
pub fn main() -> ! {
    loop {}
}

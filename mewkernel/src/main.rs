#![no_std]
#![no_main]

extern crate mewkernel_rt;

use core::arch::asm;

use mewkernel_hal::{gpio, systick, usart};
use mewkernel_rbuf::RingBuffer;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe fn sv_call_handler() {
    loop {}
}

const NVIC_ISER_BASE: usize = 0xE000_E100;

unsafe fn enable_nvic(index: u8) {
    let nvic_iser_index = (index / 32) as usize;
    let nvic_bit = index % 32;
    let nvic_iser = (NVIC_ISER_BASE + (nvic_iser_index * 4)) as *mut usize;
    *nvic_iser |= 1 << nvic_bit;
}

#[no_mangle]
static mut UART_TX_BUFFER: RingBuffer<256> = RingBuffer::new();

#[no_mangle]
pub unsafe fn usart2_interrupt_handler() {
    let usart_if = usart::Interface::Usart2;
    if usart_if.get_status().is_transmission_complete() {
        if let Some(value) = UART_TX_BUFFER.read() {
            usart_if.write_byte(value);
        } else {
            usart_if.set_disable(usart::Control::TransmissionCompleteInterrupt);
        }
    }
}

#[no_mangle]
pub unsafe fn sys_tick_handler() {
    if !UART_TX_BUFFER.is_empty() {
        let usart_if = usart::Interface::Usart2;
        usart_if.set_enable(usart::Control::TransmissionCompleteInterrupt);
    }
}

const USART2_INTERRUPT_ID: u8 = 38;

#[no_mangle]
pub unsafe fn main() -> ! {
    gpio::Port::A.enable_clock(true);
    enable_nvic(USART2_INTERRUPT_ID);

    systick::enable();

    let led_pin = gpio::Pin::from_port_and_pin(gpio::Port::A, 5);
    led_pin.set_mode(gpio::Mode::Output);

    let usart_if = usart::Interface::Usart2;
    usart_if.enable_clock(true);
    usart_if.set_word_length(usart::WordLength::Bits8);
    usart_if.set_stop_bits(usart::StopBits::Bits1);
    usart_if.set_oversampling(usart::Oversampling::By16);
    usart_if.set_parity(usart::Parity::Odd);
    usart_if.set_baud_rate::<16_000_000>(115200);
    usart_if.set_enable_disable(usart::Control::Usart | usart::Control::Transmitter);

    let usart2_tx_pin = gpio::Pin::from_port_and_pin(gpio::Port::A, 2);
    usart2_tx_pin.set_mode(gpio::Mode::AlternateFunction);
    usart2_tx_pin.set_type(gpio::Type::PushPull);
    usart2_tx_pin.set_speed(gpio::Speed::High);
    usart2_tx_pin.set_pull_up_down(gpio::PullUpDown::None);
    usart2_tx_pin.set_alternate_function(7);

    while let Err(_) = UART_TX_BUFFER.write_slice(b"Board initialized.\n\r") {}

    let mut led_state = false;
    loop {
        led_pin.set(led_state);
        led_state = !led_state;
        let message = if led_state {
            b"Turned LED on.\n\r" as &[u8]
        } else {
            b"Turned LED off.\n\r" as &[u8]
        };

        while let Err(_) = UART_TX_BUFFER.write_slice(message) {}

        for _ in 0..1_000_000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

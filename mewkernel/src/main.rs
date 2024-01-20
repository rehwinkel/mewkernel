#![no_std]
#![no_main]

extern crate mewkernel_rt;

use core::arch::asm;
use core::cell::{OnceCell, UnsafeCell};

use mewkernel_hal::usart;
use mewkernel_hal::{gpio, systick};
use ringbuf::{ring_buffer::RbBase, Rb, StaticRb};

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

static mut BUFFERED_USART: OnceCell<UnsafeCell<UsartBuffer>> = OnceCell::new();

struct UsartBuffer {
    tx_buffer: StaticRb<u8, 256>,
    rx_buffer: StaticRb<u8, 256>,
}

impl UsartBuffer {
    unsafe fn global() -> &'static UnsafeCell<Self> {
        BUFFERED_USART.get_or_init(|| {
            UnsafeCell::new(UsartBuffer {
                tx_buffer: StaticRb::default(),
                rx_buffer: StaticRb::default(),
            })
        })
    }

    fn rx() -> &'static mut StaticRb<u8, 256> {
        unsafe { &mut Self::global().get().as_mut().unwrap().rx_buffer }
    }

    fn tx() -> &'static mut StaticRb<u8, 256> {
        unsafe { &mut Self::global().get().as_mut().unwrap().tx_buffer }
    }
}

#[no_mangle]
pub unsafe fn usart2_interrupt_handler() {
    let usart_if = usart::Interface::Usart2;
    if usart_if.get_status().is_transmission_complete() {
        if let Some(value) = UsartBuffer::tx().pop() {
            usart_if.write_byte(value);
        } else {
            usart_if.set_disable(usart::Control::TransmissionCompleteInterrupt);
        }
    }
    if usart_if.get_status().is_receive_register_not_empty() {
        if !UsartBuffer::rx().is_full() {
            let value = usart_if.read_byte();
            UsartBuffer::rx().push(value).unwrap();
        }
    }
}

#[no_mangle]
pub unsafe fn sys_tick_handler() {
    if !UsartBuffer::tx().is_empty() {
        let usart_if = usart::Interface::Usart2;
        usart_if.set_enable(usart::Control::TransmissionCompleteInterrupt);
    }
}

const USART2_INTERRUPT_ID: u8 = 38;

unsafe fn read_byte_sync() -> u8 {
    while UsartBuffer::rx().is_empty() {}
    UsartBuffer::rx().pop().unwrap()
}

#[no_mangle]
#[inline(never)]
unsafe fn read_until_line<'a>(line: &'a mut [u8]) -> Option<&'a [u8]> {
    let mut index = 0;
    while index < line.len() {
        let value = read_byte_sync();
        line[index] = value;
        index += 1;
        if value == b'\r' {
            return Some(&line[0..index]);
        }
    }
    None
}

fn push_slice_wait<R: Rb<u8>>(buffer: &mut R, slice: &[u8]) {
    while slice.len() > buffer.free_len() {}
    buffer.push_slice(slice);
}

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
    usart_if.set_enable_disable(
        usart::Control::Usart
            | usart::Control::Transmitter
            | usart::Control::Receiver
            | usart::Control::ReceiveRegisterNotEmptyInterrupt,
    );

    let usart2_tx_pin = gpio::Pin::from_port_and_pin(gpio::Port::A, 2);
    usart2_tx_pin.set_mode(gpio::Mode::AlternateFunction);
    usart2_tx_pin.set_type(gpio::Type::PushPull);
    usart2_tx_pin.set_speed(gpio::Speed::High);
    usart2_tx_pin.set_pull_up_down(gpio::PullUpDown::None);
    usart2_tx_pin.set_alternate_function(7);
    let usart2_rx_pin = gpio::Pin::from_port_and_pin(gpio::Port::A, 3);
    usart2_rx_pin.set_mode(gpio::Mode::AlternateFunction);
    usart2_rx_pin.set_type(gpio::Type::PushPull);
    usart2_rx_pin.set_speed(gpio::Speed::High);
    usart2_rx_pin.set_pull_up_down(gpio::PullUpDown::None);
    usart2_rx_pin.set_alternate_function(7);

    push_slice_wait(UsartBuffer::tx(), b"Board initialized.\n\r");

    let mut led_state = false;
    loop {
        led_pin.set(led_state);
        led_state = !led_state;
        let message = if led_state {
            b"Turned LED on.\n\r" as &[u8]
        } else {
            b"Turned LED off.\n\r" as &[u8]
        };

        let line_buffer = &mut [0; 32];
        loop {
            if let Some(line) = read_until_line(line_buffer) {
                push_slice_wait(UsartBuffer::tx(), b"Read line: ");
                push_slice_wait(UsartBuffer::tx(), line);
                push_slice_wait(UsartBuffer::tx(), b"\r\n");
                break;
            }
        }

        push_slice_wait(UsartBuffer::tx(), message);

        for _ in 0..1_000_000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

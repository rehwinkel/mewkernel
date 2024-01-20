use core::ops::BitOr;

use crate::util::SetBits;

pub enum Interface {
    Usart1,
    Usart2,
    Usart3,
    Uart4,
    Uart5,
    Usart6,
}

#[derive(PartialEq, Eq)]
pub enum Oversampling {
    By16,
    By8,
}

#[derive(PartialEq, Eq)]
pub enum WordLength {
    Bits8,
    Bits9,
}

#[derive(PartialEq, Eq)]
pub enum Parity {
    Odd,
    Even,
}

pub enum StopBits {
    Bits05 = 1,
    Bits1 = 0,
    Bits15 = 3,
    Bits2 = 2,
}

#[derive(Copy, Clone)]
pub enum Control {
    Usart = 1 << 13,
    Transmitter = 1 << 3,
    Receiver = 1 << 2,
    ParityControl = 1 << 10,
    TransmissionCompleteInterrupt = 1 << 6,
    ReceiveRegisterNotEmptyInterrupt = 1 << 5,
}

#[derive(Copy, Clone)]
pub struct Controls(usize);

impl Control {
    fn get_bitmask() -> usize {
        (Self::Usart
            | Self::Transmitter
            | Self::Receiver
            | Self::ParityControl
            | Self::TransmissionCompleteInterrupt
            | Self::ReceiveRegisterNotEmptyInterrupt)
            .0
    }
}

impl BitOr for Control {
    type Output = Controls;

    fn bitor(self, rhs: Self) -> Self::Output {
        Controls(self as usize | rhs as usize)
    }
}

impl BitOr<Control> for Controls {
    type Output = Controls;

    fn bitor(self, rhs: Control) -> Self::Output {
        Controls(self.0 | rhs as usize)
    }
}

pub trait AsControls {
    fn as_controls(&self) -> Controls;
}

impl AsControls for Control {
    fn as_controls(&self) -> Controls {
        Controls(*self as usize)
    }
}

impl AsControls for Controls {
    fn as_controls(&self) -> Controls {
        *self
    }
}

const RCC_BASE: usize = 0x4002_3800;
const RCC_APB1ENR: usize = RCC_BASE + 0x40;
const RCC_APB2ENR: usize = RCC_BASE + 0x44;

const fn compute_usartdiv(clock_freq: usize, baud: usize, over8: usize) -> usize {
    let divisor = baud * 8 * (2 - over8);
    let mantissa = clock_freq / divisor;
    let remainder = clock_freq % divisor;
    let fraction = (remainder * 16 + (divisor / 2)) / divisor;
    (mantissa << 4) | fraction
}

pub struct Status(usize);

impl Status {
    fn from(value: usize) -> Self {
        Self(value & 0x1FF)
    }

    pub fn is_transmission_complete(&self) -> bool {
        (self.0 & (1 << 6)) > 0
    }

    pub fn is_receive_register_not_empty(&self) -> bool {
        (self.0 & (1 << 5)) > 0
    }

    pub fn is_overrun_error(&self) -> bool {
        (self.0 & (1 << 3)) > 0
    }
}

impl Interface {
    pub fn get_register(&self, offset: usize) -> *mut usize {
        let base: usize = match self {
            Interface::Usart6 => 0x4001_1400,
            Interface::Usart1 => 0x4001_1000,
            Interface::Uart5 => 0x4000_5000,
            Interface::Uart4 => 0x4000_4C00,
            Interface::Usart3 => 0x4000_4800,
            Interface::Usart2 => 0x4000_4400,
        };
        (base + offset) as *mut usize
    }

    pub unsafe fn enable_clock(&self, enable: bool) {
        let rcc_register = match self {
            Interface::Usart6 | Interface::Usart1 => RCC_APB2ENR as *mut usize,
            _ => RCC_APB1ENR as *mut usize,
        };
        let rcc_bit = match self {
            Interface::Usart6 => 5,
            Interface::Usart1 => 4,
            Interface::Usart2 => 17,
            Interface::Usart3 => 18,
            Interface::Uart4 => 19,
            Interface::Uart5 => 20,
        };
        rcc_register.set_bits(rcc_bit, 1, enable as usize);
    }

    fn get_control_register_1(&self) -> *mut usize {
        self.get_register(0xc)
    }

    fn get_control_register_2(&self) -> *mut usize {
        self.get_register(0x10)
    }

    fn get_data_register(&self) -> *mut usize {
        self.get_register(0x4)
    }

    fn get_status_register(&self) -> *mut usize {
        self.get_register(0x0)
    }

    pub unsafe fn set_oversampling(&self, oversampling: Oversampling) {
        let control_register_1 = self.get_control_register_1();
        control_register_1.set_bits(15, 1, (oversampling == Oversampling::By8) as usize);
    }

    pub unsafe fn set_word_length(&self, word_length: WordLength) {
        let control_register_1 = self.get_control_register_1();
        control_register_1.set_bits(12, 1, (word_length == WordLength::Bits9) as usize);
    }

    pub unsafe fn set_enable_disable<T: AsControls>(&self, controls: T) {
        let control_register_1 = self.get_control_register_1();
        control_register_1.write_volatile(
            (control_register_1.read_volatile() & !Control::get_bitmask())
                | controls.as_controls().0,
        );
    }

    pub unsafe fn set_parity(&self, parity: Parity) {
        let control_register_1 = self.get_control_register_1();
        control_register_1.set_bits(9, 1, (parity == Parity::Odd) as usize);
    }

    pub unsafe fn set_baud_rate<const CLOCK: usize>(&self, baud: usize) {
        let control_register_1 = self.get_control_register_1();
        let over8 = ((control_register_1.read_volatile() & (1 << 15)) > 0) as usize;
        let usartdiv = compute_usartdiv(CLOCK, baud, over8);
        let baud_rate_register = self.get_register(0x8);
        baud_rate_register.write_volatile(usartdiv);
    }

    pub unsafe fn set_stop_bits(&self, stop_bits: StopBits) {
        let control_register_2 = self.get_control_register_2();
        control_register_2.set_bits(12, 2, stop_bits as usize);
    }

    pub unsafe fn write_byte(&self, ch: u8) {
        let data_register = self.get_data_register();
        data_register.write_volatile(ch as usize);
    }

    pub unsafe fn read_byte(&self) -> u8 {
        let data_register = self.get_data_register();
        data_register.read_volatile() as u8
    }

    pub unsafe fn get_status(&self) -> Status {
        let status_register = self.get_status_register();
        Status::from(status_register.read_volatile())
    }

    pub unsafe fn set_disable(&self, control: Control) {
        let control_register_1 = self.get_control_register_1();
        control_register_1.write_volatile(control_register_1.read_volatile() & !(control as usize));
    }

    pub unsafe fn set_enable(&self, control: Control) {
        let control_register_1 = self.get_control_register_1();
        control_register_1.write_volatile(control_register_1.read_volatile() | (control as usize));
    }
}

use crate::util::SetBits;

const RCC_BASE: usize = 0x4002_3800;
const RCC_AHB1ENR: usize = RCC_BASE + 0x30;

#[derive(Copy, Clone)]
pub enum Port {
    A = 0,
}

pub struct Pin {
    pin: u8,
    port: Port,
}

impl Pin {
    pub fn from_port_and_pin(port: Port, pin: u8) -> Self {
        assert!(pin < 16);
        Pin { port, pin }
    }

    pub unsafe fn set_mode(&self, mode: Mode) {
        let moder = self.port.get_mode_register();
        moder.set_bits(self.pin as usize * 2, 2, mode as usize);
    }

    pub unsafe fn set_type(&self, gpio_type: Type) {
        let otyper = self.port.get_output_type_register();
        otyper.set_bits(self.pin as usize, 1, gpio_type as usize);
    }

    pub unsafe fn set_speed(&self, speed: Speed) {
        let ospeedr = self.port.get_output_speed_register();
        ospeedr.set_bits(self.pin as usize * 2, 2, speed as usize);
    }

    pub unsafe fn set(&self, value: bool) {
        let odr = self.port.get_output_data_register();
        odr.set_bits(self.pin as usize, 1, value as usize);
    }

    pub unsafe fn set_alternate_function(&self, alternate_function: u8) {
        assert!(alternate_function < 16);
        let afr = if self.pin < 8 {
            self.port.get_alternate_function_low_register()
        } else {
            self.port.get_alternate_function_high_register()
        };
        let afr_index = (self.pin % 8) as usize;
        afr.set_bits(afr_index * 4, 4, alternate_function as usize);
    }

    pub unsafe fn set_pull_up_down(&self, pull_up_down: PullUpDown) {
        let pupdr = self.port.get_pull_up_down_register();
        pupdr.set_bits(self.pin as usize * 2, 2, pull_up_down as usize);
    }
}

#[derive(Copy, Clone)]
pub enum Mode {
    Input = 0,
    Output = 1,
    AlternateFunction = 2,
    Analog = 3,
}

#[derive(Copy, Clone)]
pub enum PullUpDown {
    None = 0,
    PullUp = 1,
    PullDown = 2,
}

#[derive(Copy, Clone)]
pub enum Type {
    PushPull = 0,
    OpenDrain = 1,
}

#[derive(Copy, Clone)]
pub enum Speed {
    Low = 0,
    Medium = 1,
    Fast = 2,
    High = 3,
}

impl Port {
    fn get_register(&self, offset: usize) -> *mut usize {
        let base: usize = match self {
            Port::A => 0x4002_0000,
        };
        (base + offset) as *mut usize
    }

    fn get_output_data_register(&self) -> *mut usize {
        self.get_register(0x14)
    }

    fn get_output_type_register(&self) -> *mut usize {
        self.get_register(0x4)
    }

    fn get_output_speed_register(&self) -> *mut usize {
        self.get_register(0x8)
    }

    fn get_pull_up_down_register(&self) -> *mut usize {
        self.get_register(0xC)
    }

    fn get_mode_register(&self) -> *mut usize {
        self.get_register(0x0)
    }

    fn get_alternate_function_low_register(&self) -> *mut usize {
        self.get_register(0x20)
    }

    fn get_alternate_function_high_register(&self) -> *mut usize {
        self.get_register(0x24)
    }

    pub unsafe fn enable_clock(&self, enable: bool) {
        let rcc_ahb1enr = RCC_AHB1ENR as *mut usize;
        rcc_ahb1enr.set_bits(*self as usize, 1, enable as usize);
    }
}

#![no_std]

const KIB: usize = 1024;
const RAM_START: usize = 0x2000_0000;
const RAM_SIZE: usize = 128 * KIB;

const INITIAL_STACK_POINTER: usize = RAM_START + RAM_SIZE;

#[derive(Copy, Clone)]
pub union Vector {
    exc_handler: unsafe extern "C" fn() -> !,
    handler: unsafe extern "C" fn(),
    value: usize,
}

const USART2_INTERRUPT_ID: u8 = 38;

#[link_section = ".isr_vector"]
#[no_mangle]
static ISR_VECTOR: [Vector; 16 + 96] = {
    let mut arr = [Vector { value: 0 }; 16 + 96];
    arr[0] = Vector {
        value: INITIAL_STACK_POINTER,
    };
    arr[1] = Vector {
        exc_handler: reset_handler,
    };
    arr[2] = Vector {
        exc_handler: default_handler,
    };
    arr[3] = Vector {
        exc_handler: default_handler,
    };
    arr[4] = Vector {
        exc_handler: default_handler,
    };
    arr[5] = Vector {
        exc_handler: default_handler,
    };
    arr[6] = Vector {
        exc_handler: default_handler,
    };
    arr[11] = Vector {
        handler: _c_sv_call_handler,
    };
    arr[12] = Vector {
        exc_handler: default_handler,
    };
    arr[14] = Vector {
        exc_handler: default_handler,
    };
    arr[15] = Vector {
        handler: _c_sys_tick_handler,
    };
    arr[(USART2_INTERRUPT_ID as usize) + 16] = Vector {
        handler: _c_usart2_interrupt_handler,
    };
    arr
};

unsafe extern "C" fn _c_sv_call_handler() {
    extern "Rust" {
        fn sv_call_handler();
    }

    sv_call_handler();
}

unsafe extern "C" fn _c_sys_tick_handler() {
    extern "Rust" {
        fn sys_tick_handler();
    }

    sys_tick_handler();
}

unsafe extern "C" fn _c_usart2_interrupt_handler() {
    extern "Rust" {
        fn usart2_interrupt_handler();
    }

    usart2_interrupt_handler();
}

#[no_mangle]
unsafe extern "C" fn reset_handler() -> ! {
    extern "C" {
        static mut _sbss: usize;
        static mut _ebss: usize;
        static mut _sdata: usize;
        static mut _edata: usize;
        static mut _etext: usize;
    }

    // Copy .data from flash to sram
    let data_size = (&_edata as *const usize as usize) - (&_sdata as *const usize as usize);
    let flash_data = &_etext as *const usize as *const u8;
    let sram_data = &_sdata as *const usize as *mut u8;
    core::ptr::copy_nonoverlapping(flash_data, sram_data, data_size);

    // Clear .bss to zero
    let bss_size = (&_ebss as *const usize as usize) - (&_sbss as *const usize as usize);
    let bss_data = &_sbss as *const usize as *mut u8;
    core::ptr::write_bytes(bss_data, 0, bss_size);

    extern "Rust" {
        fn main() -> !;
    }

    main();
}

unsafe extern "C" fn default_handler() -> ! {
    loop {}
}

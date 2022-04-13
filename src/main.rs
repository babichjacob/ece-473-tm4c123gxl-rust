#![no_std]
#![no_main]

use core::ptr;

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use test_cortex_m4_rust::{Function, Bit, ReadablePinSetup, WritablePinSetup, PortSetup, Port, setup_board, H, L};
// use test_cortex_m4_rust::setup_board;

// Begin .h file contents

// GPIO registers (PORTF)
const GPIO_PORTF_DATA_R: *mut u32 = 0x400253FC as *mut u32;
const GPIO_PORTF_DIR_R: *mut u32 = 0x40025400 as *mut u32;
const GPIO_PORTF_AFSEL_R: *mut u32 = 0x40025420 as *mut u32;
const GPIO_PORTF_PUR_R: *mut u32 = 0x40025510 as *mut u32;
const GPIO_PORTF_DEN_R: *mut u32 = 0x4002551C as *mut u32;
const GPIOLOCK_PORT_F: *mut u32 = 0x40025520 as *mut u32;
const GPIOCR_PORT_F: *mut u32 = 0x40025524 as *mut u32;
const GPIOAMSEL_PORT_F: *mut u32 = 0x40025528 as *mut u32;
const GPIO_PORTF_PCTL_R: *mut u32 = 0x4002552C as *mut u32;

const SYSCTL_RCGCPIO_R: *mut u32 = 0x400FE608 as *mut u32;

// Begin starter project contents

const GPIO_LOCK_KEY: *mut u32 = 0x4C4F434B as *mut u32;
const PF0: *mut u32 = 0x40025004 as *mut u32;
const PF4: *mut u32 = 0x40025040 as *mut u32;
const SWITCHES: *mut u32 = 0x40025044 as *mut u32;

const SW1: u8 = 0b0001_0000;
const SW2: u8 = 0b0000_0001;

const SYSCTL_RCGC2_GPIOF: *mut u32 = 0x00000020 as *mut u32;

const BLACK: u8 = 0b0000_0000;
const RED: u8 = 0b0000_0010;
const GREEN: u8 = 0b0000_1000;
const BLUE: u8 = 0b0000_0100;

fn setup_port_f() {
    // 1) activate clock for Port F
    // unsafe {
    //     ptr::write_volatile(
    //         SYSCTL_RCGCPIO_R,
    //         ptr::read_volatile(SYSCTL_RCGCPIO_R) | 0x00_00_00_20,
    //     );
    // }

    // Delay
    // for _ in 0u8..2u8 {}

    // 2) unlock GPIO Port F
    // unsafe {
    //     ptr::write_volatile(GPIOLOCK_PORT_F, 0x4C4F434B);
    //     // allow changes to PF4-0
    //     // only PF0 needs to be unlocked, other bits can't be locked
    //     ptr::write_volatile(GPIOCR_PORT_F, 0b0001_1111);
    // }

    // 3) disable analog on PF
    // unsafe { ptr::write_volatile(GPIOAMSEL_PORT_F, 0x00) }

    // 4) PCTL GPIO on PF4-0
    // unsafe {
    //     ptr::write_volatile(GPIO_PORTF_PCTL_R, 0x00000000);
    // }

    // 5) PF4,PF0 in, PF3-1 out
    // unsafe {
    //     ptr::write_volatile(GPIO_PORTF_DIR_R, 0x0E);
    // }
    // 6) disable alt funct on PF7-0
    // unsafe {
    //     ptr::write_volatile(GPIO_PORTF_AFSEL_R, 0x00);
    // }
    // enable pull-up on PF0 and PF4
    unsafe {
        ptr::write_volatile(GPIO_PORTF_PUR_R, 0x11);
    }
    // 7) enable digital I/O on PF4-0
    unsafe {
        ptr::write_volatile(GPIO_PORTF_DEN_R, 0x1F);
    }
}

fn input_from_port_f() -> u32 {
    unsafe {
        ptr::read_volatile(GPIO_PORTF_DATA_R) & u32::from(SW1 | SW2)
    }
}

fn output_to_port_f(value: u8) {
    unsafe {
        ptr::write_volatile(GPIO_PORTF_DATA_R, u32::from(value));
    }
}

#[entry]
fn main() -> ! {
    let board = setup_board();
    let port_f = board.setup_gpio_port(Port::F, PortSetup);

    let switches = port_f.setup_readable_pins(&[Bit::Zero, Bit::Four], ReadablePinSetup {
        function: Function::Digital,
    });
    let [sw1, sw2] = switches.pins();

    let mut rgb_led = port_f.setup_writable_pins(&[Bit::One, Bit::Three, Bit::Two], WritablePinSetup { function: Function::Digital });

    loop {
        match switches.read_all() {
            [L, L] => rgb_led.write_all([L, H, L]),
            [L, H] => rgb_led.write_all([L, L, H]),
            [H, L] => rgb_led.write_all([L, H, L]),
            [H, H] => rgb_led.write_all([H, L, L]),
        }
    }
}

// #[entry]
// fn main() -> ! {
//     setup_port_f();

//     loop {
//         let status = input_from_port_f();

//         match status {
//             0x00 => output_to_port_f(RED | GREEN),
//             0x01 => output_to_port_f(RED),
//             0x10 => output_to_port_f(GREEN),
//             0x11 => output_to_port_f(BLACK),
//             // Impossible case
//             _ => output_to_port_f(BLUE),
//         }
//     }
// }

// TODO: implement an extremely simple example of the task system (using a timer trigger) that is a traffic light (green -> yellow -> red -> green...)

#![no_std]
#![no_main]

use core::{arch::asm, ptr};

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use driver_and_task_library::{
    setup_board, Function, Pin, Port, PortOptions, ReadablePinOptions, UsableBoard,
    WritablePinOptions, H, L,
};

const SYSCTL_RCGC1_R: *mut u32 = 0x400FE104 as *mut u32;
const SYSCTL_RCGC2_R: *mut u32 = 0x400FE108 as *mut u32;

const UART0_DR_R: *mut u32 = 0x4000C000 as *mut u32;
const UART0_FR_R: *mut u32 = 0x4000C018 as *mut u32;
const UART0_IBRD_R: *mut u32 = 0x4000C024 as *mut u32;
const UART0_FBRD_R: *mut u32 = 0x4000C028 as *mut u32;
const UART0_LCRH_R: *mut u32 = 0x4000C02C as *mut u32;
const UART0_CTL_R: *mut u32 = 0x4000C030 as *mut u32;

const GPIO_PORTA_AFSEL_R: *mut u32 = 0x40004420 as *mut u32;
const GPIO_PORTA_DEN_R: *mut u32 = 0x4000451C as *mut u32;

/// UART0 Clock Gating Control
const SYSCTL_RCGC1_UART0: u32 = 0x00000001;
/// port A Clock Gating Control
const SYSCTL_RCGC2_GPIOA: u32 = 0x00000001;
/// UART Enable
const UART_CTL_UARTEN: u32 = 0x00000001;
/// 8 bit word length
const UART_LCRH_WLEN_8: u32 = 0x00000060;
/// UART Enable FIFOs
const UART_LCRH_FEN: u32 = 0x00000010;
/// UART Transmit FIFO Full
const UART_FR_TXFF: u32 = 0x00000020;
/// UART Receive FIFO Empty
const UART_FR_RXFE: u32 = 0x00000010;
/// Pins 0 and 1
const PINS_0_AND_1: u32 = 0b0000_0011;

fn uart0_init(board: UsableBoard) {
    unsafe {
        // activate UART0
        ptr::write_volatile(
            SYSCTL_RCGC1_R,
            ptr::read_volatile(SYSCTL_RCGC1_R) | SYSCTL_RCGC1_UART0,
        );
        // activate port A
        // ptr::write_volatile(SYSCTL_RCGC2_R, ptr::read_volatile(SYSCTL_RCGC2_R) | SYSCTL_RCGC2_GPIOA);
        // ^ commented in favor of v
        board.setup_gpio_port(Port::A, PortOptions);

        // disable UART while setting it up
        ptr::write_volatile(
            UART0_CTL_R,
            ptr::read_volatile(UART0_CTL_R) & !UART_CTL_UARTEN,
        );

        // ignore: // IBRD = int(50,000,000 / (16 * 115,200)) = int(27.1267)
        // IBRD = int(16,000,000 / (16 * 115,200)) = int(8.680)
        // ptr::write_volatile(UART0_IBRD_R, 8);
        ptr::write_volatile(UART0_IBRD_R, 8);

        // ignore: // FBRD = int(0.1267 * 64 + 0.5) = 8
        // FBRD = round(0.5104 * 64 ) = 33 --- that ain't the number you wrote but ok
        // ptr::write_volatile(UART0_FBRD_R, 44);
        ptr::write_volatile(UART0_FBRD_R, 44);

        // 8 bit word length (no parity bits, one stop bit, FIFOs)
        // ptr::write_volatile(UART0_LCRH_R, UART_LCRH_WLEN_8|UART_LCRH_FEN);
        // 8 bit word length (no parity bits, one stop bit, no FIFOs)
        ptr::write_volatile(UART0_LCRH_R, UART_LCRH_WLEN_8 & !UART_LCRH_FEN);

        // enable UART since it's been set up
        ptr::write_volatile(
            UART0_CTL_R,
            ptr::read_volatile(UART0_CTL_R) | UART_CTL_UARTEN,
        );

        // enable alt funct on PA1-0
        ptr::write_volatile(
            GPIO_PORTA_AFSEL_R,
            ptr::read_volatile(GPIO_PORTA_AFSEL_R) | PINS_0_AND_1,
        );
        // enable digital I/O on PA1-0
        ptr::write_volatile(
            GPIO_PORTA_DEN_R,
            ptr::read_volatile(GPIO_PORTA_AFSEL_R) | PINS_0_AND_1,
        );
    }
}

fn uart0_out_char(c: u8) {
    loop {
        let fr = unsafe { ptr::read_volatile(UART0_FR_R) };

        if (fr & UART_FR_TXFF) == 0 {
            break;
        }
    }

    unsafe {
        ptr::write_volatile(UART0_DR_R, c as u32);
    }
}

fn uart0_out_string(s: &str) {
    for c in s.bytes() {
        uart0_out_char(c);
    }
}

#[entry]
fn main() -> ! {
    let board = setup_board();
    let port_f = board.setup_gpio_port(Port::F, PortOptions);

    let switches = port_f.setup_readable_pins(
        [Pin::Zero, Pin::Four],
        ReadablePinOptions {
            function: Function::Digital,
            pull_up: Some(true),
        },
    );
    let [_sw1, _sw2] = switches.pins();

    let mut rgb_led = port_f.setup_writable_pins(
        [Pin::One, Pin::Three, Pin::Two],
        WritablePinOptions {
            function: Function::Digital,
        },
    );

    let white = [H, H, H];
    let _black = [L, L, L];

    let red = [H, L, L];
    let yellow = [H, H, L];
    let green = [L, H, L];
    let cyan = [L, H, H];
    let blue = [L, L, H];
    let magenta = [H, L, H];

    let rainbow = [red, yellow, green, cyan, blue, magenta];

    rgb_led.write_all(cyan);

    uart0_init(board);

    rgb_led.write_all(white);

    for _ in 0..2 {
        for c in [
            'H', 'a', 'y', '!', '\r', '\n', 'H', 'e', 'y', '!', '\r', '\n', 'H', 'e', 'y', '!',
            '\r', '\n', 'H', 'e', 'y', '!', '\r', '\n',
        ] {
            uart0_out_char(c as u8);
        }
    }
    uart0_out_string("Those example string!\r\n");

    loop {
        match switches.read_all() {
            [L, L] => {
                rgb_led.write_all(white);
                uart0_out_string("Hey! You're pressing the button down!\r\n");
            }
            [L, H] => rgb_led.write_all(blue),
            [H, L] => rgb_led.write_all(red),
            [H, H] => rgb_led.write_all(green),
        }

        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

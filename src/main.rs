#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use cortex_m_rt::entry;
use driver_and_task_library::{
    setup_board, Function, GPIOPort, Pin, Pull, ReadablePinOptions, UARTPort,
    UARTPortOptions, WordLength, WritablePinOptions, H, L,
};

const WHITE: [bool; 3] = [H, H, H];
const BLACK: [bool; 3] = [L, L, L];

const RED: [bool; 3] = [H, L, L];
const _YELLOW: [bool; 3] = [H, H, L];
const _GREEN: [bool; 3] = [L, H, L];
const _CYAN: [bool; 3] = [L, H, H];
const BLUE: [bool; 3] = [L, L, H];
const _MAGENTA: [bool; 3] = [H, L, H];

static _RAINBOW: [[bool; 3]; 6] = [RED, _YELLOW, _GREEN, _CYAN, BLUE, _MAGENTA];

#[entry]
fn main() -> ! {
    let mut board = setup_board();

    let mut port_f = board.setup_gpio_port(GPIOPort::F);
    let switches = port_f.setup_readable_pins(
        [Pin::Zero, Pin::Four],
        ReadablePinOptions {
            function: Function::Digital,
            pull: Pull::Up,
        },
    );
    let mut rgb_led = port_f.setup_writable_pins(
        [Pin::One, Pin::Three, Pin::Two],
        WritablePinOptions {
            function: Function::Digital,
        },
    );

    let mut port_a = board.setup_gpio_port(GPIOPort::A);
    let [uart_0_rx] = port_a
        .setup_readable_pins(
            [Pin::Zero],
            ReadablePinOptions {
                function: Function::UART,
                pull: Pull::Neither,
            },
        ).pins();
    let [mut uart_0_tx] = port_a
        .setup_writable_pins(
            [Pin::One],
            WritablePinOptions {
                function: Function::UART,
            },
        ).pins();
    let mut uart_0 = board.setup_uart_port(
        UARTPort::Zero,
        UARTPortOptions {
            baud_rate: 115_200,
            fifos: true,
            word_length: WordLength::Eight,
        },
    );

    uart_0.write_line(&mut uart_0_tx, "");
    uart_0.write_line(&mut uart_0_tx, "Program start!");

    loop {
        match switches.read_all() {
            [L, L] => rgb_led.write_all(WHITE),
            [L, H] => rgb_led.write_all(BLUE),
            [H, L] => rgb_led.write_all(RED),
            [H, H] => rgb_led.write_all(BLACK),
        }

        uart_0.write_string(&mut uart_0_tx, "What's your name? ");
        let input = uart_0.read_line(&mut uart_0_tx, &uart_0_rx);
        uart_0.write_line(&mut uart_0_tx, &format!("Good afternoon {:?}!", &input));
    }
}

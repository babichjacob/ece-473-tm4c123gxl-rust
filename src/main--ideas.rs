#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;

use my_library::{Bit, Color, setup_board};

const H: bool = true;
const L: bool = false;

#[entry]
fn main() -> ! {
    let board = setup_board();

    let port_f = board.setup_port(Port::F);

    let switches = port_f.setup_readable_pins([Bit::Zero, Bit::Four], PinSetup {
        alternate_function: false,
        analog: false,
        pullup: true,
        pctl: false,
    });

    let rgb_led = port_f.setup_writable_pins([Bit::One, Bit::Three, Bit::Two], PinSetup {
        alternate_function: false,
        analog: false,
        pctl: false,
    });

    // Integrate PWM for arbitrary color support
    let rgb_led_driver = rgb_led.driver();

    // Maybe?
    let every_5_seconds = board.time_trigger(5);

    // Example of adding tasks
    board.add_task(
        some_kind_of_task,
        10, // priority maybe?
        every_5_seconds, // trigger every 5 seconds
    );

    loop {
        match switches.read_all() {
            [L, L] => rgb_led_driver.set_color(Color::Green),
            [L, H] => rgb_led_driver.set_color(Color::Blue),
            [H, L] => rgb_led_driver.set_color(Color::Red),
            [H, H] => rgb_led_driver.set_color(Color::Black),
        }
    }
}


fn some_kind_of_task() {
    // ...
}


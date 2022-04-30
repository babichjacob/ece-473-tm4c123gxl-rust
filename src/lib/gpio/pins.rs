use crate::{
    memory,
    utils::{pins_to_bits, reverse_array},
    H, L,
};

use super::ports::Port;

#[derive(Clone, Copy)]
pub enum Pin {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}
pub enum Function {
    Analog,
    Digital,
    CAN,
    I2C,
    PWM,
    UART,
}

pub enum Pull {
    Down,
    Up,
    Neither,
}

/// Page 1351 of data sheet
pub struct ReadablePinOptions {
    pub function: Function,
    pub pull: Pull,
}
pub struct ReadablePins<const N: usize> {
    data_address: *mut u32,
    pins: [ReadablePin; N],
}
impl<const N: usize> ReadablePins<N> {
    pub fn pins(&self) -> [ReadablePin; N] {
        self.pins
    }

    pub fn read_all(&self) -> [bool; N] {
        unsafe { memory::read_bits(self.data_address, &self.pins.map(|pin| pin.pin as u32)) }
    }
}
#[derive(Clone, Copy)]
pub struct ReadablePin {
    data_address: *mut u32,
    pin: Pin,
}
impl ReadablePin {
    pub fn read(&self) -> bool {
        let current = unsafe { memory::read(self.data_address) };
        current & (1 << self.pin as u32) != 0
    }
}

pub struct WritablePinOptions {
    pub function: Function,
}
pub struct WritablePins<const N: usize> {
    data_address: *mut u32,
    pins: [WritablePin; N],
}
impl<const N: usize> WritablePins<N> {
    pub fn pins(&self) -> [WritablePin; N] {
        self.pins
    }

    pub fn read_all(&self) -> [bool; N] {
        unsafe { memory::read_bits(self.data_address, &self.pins.map(|pin| pin.pin as u32)) }
    }
    pub fn write_all(&mut self, values: [bool; N]) {
        unsafe {
            memory::write_bits(
                self.data_address,
                &self.pins.map(|pin| pin.pin as u32),
                values,
            )
        }
    }
    pub fn update_all(&mut self, updater: &dyn Fn([bool; N]) -> [bool; N]) {
        self.write_all(updater(self.read_all()));
    }

    pub fn clear_all(&mut self) {
        unsafe {
            memory::clear_bits(self.data_address, &self.pins.map(|pin| pin.pin as u32));
        }
    }
    pub fn set_all(&mut self) {
        unsafe {
            memory::set_bits(self.data_address, &self.pins.map(|pin| pin.pin as u32));
        }
    }
    pub fn toggle_all(&mut self) {
        unsafe {
            memory::toggle_bits(self.data_address, &self.pins.map(|pin| pin.pin as u32));
        }
    }
}

#[derive(Clone, Copy)]
pub struct WritablePin {
    data_address: *mut u32,
    pin: Pin,
}
impl WritablePin {
    pub fn read(&self) -> bool {
        let current = unsafe { memory::read(self.data_address) };
        current & (1 << self.pin as u32) != 0
    }
    pub fn clear(&mut self) {
        unsafe {
            memory::clear_bits(self.data_address, &[self.pin as u32]);
        }
    }
    pub fn set(&mut self) {
        unsafe {
            memory::set_bits(self.data_address, &[self.pin as u32]);
        }
    }
    pub fn toggle(&mut self) {
        unsafe {
            memory::toggle_bits(self.data_address, &[self.pin as u32]);
        }
    }
}

/// Page 684 of the data sheet for how the lock mechanism works
const UNLOCK: u32 = 0x4C4F434B;

fn setup_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    writable: bool,
    function: Function,
    pull: Pull,
) {
    // Unlock the pins
    unsafe {
        memory::write(port.lock(), UNLOCK);
    }

    // Set to output pins if output (otherwise set to input)
    if writable {
        unsafe {
            memory::set_bits(port.direction(), &pins_to_bits(&pins));
        }
    } else {
        unsafe {
            memory::clear_bits(port.direction(), &pins_to_bits(&pins));
        }
    }

    // Disable alternate function when it's not used (and enable it when it is)
    if let Function::Analog | Function::Digital = function {
        unsafe {
            memory::clear_bits(port.alternate_function_select(), &pins_to_bits(&pins));
        }
    } else {
        unsafe {
            memory::set_bits(port.alternate_function_select(), &pins_to_bits(&pins));
        }
    }

    // Configure pull-up and pull-down resistors
    match pull {
        Pull::Down => unsafe {
            memory::set_bits(port.pull_down_select(), &pins_to_bits(&pins));
        },
        Pull::Up => unsafe {
            memory::set_bits(port.pull_up_select(), &pins_to_bits(&pins));
        },
        Pull::Neither => {
            unsafe {
                memory::clear_bits(port.pull_up_select(), &pins_to_bits(&pins));
            }
            unsafe {
                memory::clear_bits(port.pull_down_select(), &pins_to_bits(&pins));
            }
        }
    }

    unsafe {
        memory::set_bits(port.commit(), &pins_to_bits(&pins));
    }

    // Enable digital function when it's needed (and disable it when it's not)
    match function {
        Function::Digital | Function::UART => unsafe {
            memory::set_bits(port.digital_enable(), &pins_to_bits(&pins));
        },
        Function::Analog => unsafe {
            memory::clear_bits(port.digital_enable(), &pins_to_bits(&pins));
        },
        _ => todo!(), // and rewrite to if let when solved
    }

    // Enable analog when it's needed (and disable it when it's not)
    if let Function::Analog = function {
        unsafe {
            memory::set_bits(port.analog_mode_select(), &pins_to_bits(&pins));
        }
    } else {
        unsafe {
            memory::clear_bits(port.analog_mode_select(), &pins_to_bits(&pins));
        }
    }

    // Table 10-2 on page 650-651 of data sheet
    let digital_function = match function {
        Function::Analog => None,
        Function::Digital => Some([L, L, L, L]),
        Function::CAN => Some([H, L, L, L]),
        Function::I2C => Some([L, L, H, H]),
        Function::PWM => Some([L, H, L, H]),
        Function::UART => Some([L, L, L, H]),
    };
    if let Some(array) = digital_function {
        let port_control_values = reverse_array(array);

        for pin in pins {
            let mut memory_bits = [0; 4];

            let min = (pin as u32) * 4;
            let max = min + 3;
            let range = min..=max;

            for (i, memory_bit) in range.enumerate() {
                memory_bits[i] = memory_bit;
            }

            unsafe {
                memory::write_bits(port.port_control(), &memory_bits, port_control_values);
            }
        }
    }
    
    unsafe {
        memory::write(port.lock(), 0);
    }
}

pub fn setup_readable_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    options: ReadablePinOptions,
) -> ReadablePins<N> {
    setup_pins(port, pins, false, options.function, options.pull);

    let data_address = port.data(&pins);
    let pins: [ReadablePin; N] = pins.map(|pin| ReadablePin { data_address, pin });
    ReadablePins { data_address, pins }
}

pub fn setup_writable_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    options: WritablePinOptions,
) -> WritablePins<N> {
    setup_pins(port, pins, true, options.function, Pull::Neither);

    let data_address = port.data(&pins);
    let pins: [WritablePin; N] = pins.map(|pin| WritablePin { data_address, pin });
    WritablePins { data_address, pins }
}

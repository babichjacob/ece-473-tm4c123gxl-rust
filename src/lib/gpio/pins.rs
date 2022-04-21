use crate::{memory, H, L};

use super::ports::Port;

fn reverse_array<const N: usize, T: Default + Copy>(array: [T; N]) -> [T; N] {
    let mut result: [T; N] = [<T>::default(); N];

    for (out_index, in_index) in (0..N).rev().enumerate() {
        result[out_index] = array[in_index];
    }

    result
}

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

fn pins_to_bits<const N: usize>(pins: &[Pin; N]) -> [u32; N] {
    pins.map(|pin| pin as u32)
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
        unsafe { memory::read_bits(self.data_address, &self.pins.map(|pin| pin.bit as u32)) }
    }
}
#[derive(Clone, Copy)]
pub struct ReadablePin {
    data_address: *mut u32,
    bit: Pin,
}
impl ReadablePin {
    pub fn read(&self) -> bool {
        let current = unsafe { memory::read(self.data_address) };
        current & (1 << self.bit as u32) != 0
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
        unsafe { memory::read_bits(self.data_address, &self.pins.map(|pin| pin.bit as u32)) }
    }
    pub fn write_all(&mut self, values: [bool; N]) {
        unsafe {
            memory::write_bits(
                self.data_address,
                &self.pins.map(|pin| pin.bit as u32),
                values,
            )
        }
    }
    pub fn update_all<Updater: Fn([bool; N]) -> [bool; N]>(&mut self, updater: Updater) {
        self.write_all(updater(self.read_all()));
    }

    pub fn clear_all(&mut self) {
        unsafe {
            memory::clear_bits(self.data_address, &self.pins.map(|pin| pin.bit as u32));
        }
    }
    pub fn set_all(&mut self) {
        unsafe {
            memory::set_bits(self.data_address, &self.pins.map(|pin| pin.bit as u32));
        }
    }
    pub fn toggle_all(&mut self) {
        unsafe {
            memory::toggle_bits(self.data_address, &self.pins.map(|pin| pin.bit as u32));
        }
    }
}

#[derive(Clone, Copy)]
pub struct WritablePin {
    data_address: *mut u32,
    bit: Pin,
}
impl WritablePin {
    pub fn read(&self) -> bool {
        let current = unsafe { memory::read(self.data_address) };
        current & (1 << self.bit as u32) != 0
    }
    pub fn clear(&mut self) {
        unsafe {
            memory::clear_bits(self.data_address, &[self.bit as u32]);
        }
    }
    pub fn set(&mut self) {
        unsafe {
            memory::set_bits(self.data_address, &[self.bit as u32]);
        }
    }
    pub fn toggle(&mut self) {
        unsafe {
            memory::toggle_bits(self.data_address, &[self.bit as u32]);
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

        memory::set_bits(port.commit(), &pins_to_bits(&pins));
    }

    // Disable analog when it's not selected (and enable analog if it is)
    match function {
        Function::Analog => unsafe {
            memory::set_bits(port.analog_mode_select(), &pins_to_bits(&pins));
        },
        _ => unsafe {
            memory::clear_bits(port.analog_mode_select(), &pins_to_bits(&pins));
        },
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

    let function_values = reverse_array(match function {
        Function::Analog => todo!(),
        Function::Digital => [L, L, L, L],
        Function::CAN => [H, L, L, L],
        Function::I2C => [L, L, H, H],
        Function::PWM => [L, H, L, H],
        Function::UART => [L, L, L, H],
    });
    for pin in pins {
        let mut memory_bits = [0; 4];

        let min = (pin as u32) * 4;
        let max = min + 3;
        let range = min..=max;

        for (i, memory_bit) in range.enumerate() {
            memory_bits[i] = memory_bit;
        }

        unsafe {
            memory::write_bits(port.port_control(), &memory_bits, function_values);
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

    // TODO: check page 671 or 682 (+ more prob) for a table showing initial pin states

    // Disable alternate function when it's not used (and enable it when it is)
    match function {
        Function::Analog | Function::Digital => unsafe {
            memory::clear_bits(port.alternate_function_select(), &pins_to_bits(&pins));
        },
        _ => unsafe {
            memory::set_bits(port.alternate_function_select(), &pins_to_bits(&pins));
        },
    }

    // Enable digital function when it's needed (and disable it when it's not)
    match function {
        Function::Digital | Function::UART => unsafe {
            memory::set_bits(port.digital_enable(), &pins_to_bits(&pins));
        },
        Function::Analog => unsafe {
            memory::clear_bits(port.digital_enable(), &pins_to_bits(&pins));
        },
        _ => todo!(),
    }
}

pub fn setup_readable_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    options: ReadablePinOptions,
) -> ReadablePins<N> {
    setup_pins(port, pins, false, options.function, options.pull);

    let data_address = port.data(&pins);
    let pins: [ReadablePin; N] = pins.map(|bit| ReadablePin { data_address, bit });
    ReadablePins { data_address, pins }
}

pub fn setup_writable_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    options: WritablePinOptions,
) -> WritablePins<N> {
    setup_pins(port, pins, true, options.function, Pull::Neither);

    let data_address = port.data(&pins);
    let pins: [WritablePin; N] = pins.map(|pin| WritablePin {
        data_address,
        bit: pin,
    });
    WritablePins { data_address, pins }
}

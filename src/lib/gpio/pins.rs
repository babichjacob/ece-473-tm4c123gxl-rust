use crate::{memory, L};

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

/// Page 1351 of data sheet
pub struct ReadablePinOptions {
    pub function: Function,
    pub pull_up: Option<bool>,
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

fn setup_pins() {
    todo!();
}

pub fn setup_readable_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    options: ReadablePinOptions,
) -> ReadablePins<N> {
    // Unlock the pins
    unsafe {
        memory::write(port.lock(), UNLOCK);

        memory::set_bits(port.commit(), &pins.map(|bit| bit as u32));
    }

    // Disable analog when it's not selected (and enable analog if it is)
    match options.function {
        Function::Analog => unsafe {
            memory::set_bits(port.analog_mode_select(), &pins.map(|bit| bit as u32));
        },
        _ => unsafe {
            memory::clear_bits(port.analog_mode_select(), &pins.map(|bit| bit as u32));
        },
    }

    unsafe {
        memory::clear_bits(port.direction(), &pins.map(|bit| bit as u32));
    }

    for pin in pins {
        let mut memory_bits = [0; 4];

        let min = (pin as u32) * 4;
        let max = min + 4;
        let range = min..max;

        for (i, memory_bit) in range.enumerate() {
            memory_bits[i] = memory_bit;
        }

        let values = match options.function {
            Function::Analog => todo!(),
            Function::Digital => [L, L, L, L],
            Function::CAN => todo!(),
            Function::I2C => todo!(),
            Function::PWM => todo!(),
            Function::UART => todo!(),
        };
        unsafe {
            memory::write_bits(port.port_control(), &memory_bits, values);
        }
    }

    // Configure pull-up and pull-down resistors
    match options.pull_up {
        Some(true) => unsafe {
            memory::set_bits(port.pull_up_select(), &pins.map(|bit| bit as u32));
        },
        Some(false) => unsafe {
            memory::set_bits(port.pull_down_select(), &pins.map(|bit| bit as u32));
        },
        None => {
            unsafe {
                memory::clear_bits(port.pull_up_select(), &pins.map(|bit| bit as u32));
            }
            unsafe {
                memory::clear_bits(port.pull_down_select(), &pins.map(|bit| bit as u32));
            }
        }
    }

    match options.function {
        Function::Digital => unsafe {
            memory::set_bits(port.digital_enable(), &pins.map(|bit| bit as u32));
        },
        Function::Analog => unsafe {
            memory::clear_bits(port.digital_enable(), &pins.map(|bit| bit as u32));
        },
        _ => todo!(),
    }

    let data_address = port.data(&pins);

    let pins: [ReadablePin; N] = pins.map(|bit| ReadablePin { data_address, bit });

    ReadablePins { data_address, pins }
}

pub fn setup_writable_pins<const N: usize>(
    port: Port,
    pins: [Pin; N],
    options: WritablePinOptions,
) -> WritablePins<N> {
    // Unlock the pins
    unsafe {
        memory::write(port.lock(), UNLOCK);

        memory::set_bits(port.commit(), &pins.map(|bit| bit as u32));
    }

    // Disable analog when it's not selected (and enable analog if it is)
    match options.function {
        Function::Analog => unsafe {
            memory::set_bits(port.analog_mode_select(), &pins.map(|bit| bit as u32));
        },
        _ => unsafe {
            memory::clear_bits(port.analog_mode_select(), &pins.map(|bit| bit as u32));
        },
    }

    unsafe {
        memory::set_bits(port.direction(), &pins.map(|bit| bit as u32));
    }

    for pin in pins {
        let mut memory_bits = [0; 4];

        let min = (pin as u32) * 4;
        let max = min + 3;
        let range = min..=max;

        for (i, memory_bit) in range.enumerate() {
            memory_bits[i] = memory_bit;
        }

        let values = match options.function {
            Function::Analog => todo!(),
            Function::Digital => [L, L, L, L],
            Function::CAN => todo!(),
            Function::I2C => todo!(),
            Function::PWM => todo!(),
            Function::UART => todo!(),
        };
        unsafe {
            memory::write_bits(port.port_control(), &memory_bits, values);
        }
    }

    // TODO: check page 671 or 682 (+ more prob) for a table showing initial pin states

    // TODO: finish

    match options.function {
        Function::Analog | Function::Digital => unsafe {
            memory::clear_bits(
                port.alternate_function_select(),
                &pins.map(|bit| bit as u32),
            );
        },
        _ => unsafe {
            memory::set_bits(
                port.alternate_function_select(),
                &pins.map(|bit| bit as u32),
            );
        },
    }

    match options.function {
        Function::Digital => unsafe {
            memory::set_bits(port.digital_enable(), &pins.map(|bit| bit as u32));
        },
        Function::Analog => unsafe {
            memory::clear_bits(port.digital_enable(), &pins.map(|bit| bit as u32));
        },
        _ => todo!(),
    }

    let data_address = port.data(&pins);

    let pins: [WritablePin; N] = pins.map(|pin| WritablePin { data_address, bit: pin });

    WritablePins { data_address, pins }
}

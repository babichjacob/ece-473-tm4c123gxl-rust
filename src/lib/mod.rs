#![no_std]

mod memory;
mod registers;

pub const H: bool = true;
pub const L: bool = false;

pub struct Board;
// TODO: check page 704 for timers
// TODO: impl Drop trait so that tasks all run before the main function ends?

impl Board {
    pub fn setup_gpio_port(&self, port: Port, options: PortSetup) -> PortIO {
        let port_io = PortIO { port };

        unsafe {
            memory::set_bits(
                registers::system::RCGCGPIO,
                &[port_io.run_mode_clock_gate_control() as u32],
            );
        }

        port_io
    }
}

// Page 684 of the data sheet for how the lock mechanism works
const UNLOCK: u32 = 0x4C4F434B;

pub enum Port {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub struct PortSetup;

pub struct PortIO {
    port: Port,
}

// TODO: refactor to just be self.base() + offset all the time - no matching
impl PortIO {
    /// The memory address of the alternate function select (AFSEL) register for this port
    fn alternate_function_select(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::afsel::PORT_A,
            Port::F => registers::gpio::afsel::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the analog mode select (AMSEL) register for this port
    fn analog_mode_select(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::amsel::PORT_A,
            Port::F => registers::gpio::amsel::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the commit (CR register for this port
    fn commit(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::cr::PORT_A,
            Port::F => registers::gpio::cr::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the data (DATA) register for this port
    fn data(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::data::PORT_A,
            Port::F => registers::gpio::data::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the direction (DIR) register for this port
    fn direction(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::dir::PORT_A,
            Port::F => registers::gpio::dir::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the lock (LOCK) register
    fn lock(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::lock::PORT_A,
            Port::F => registers::gpio::lock::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the port control (PCTL) register for this port
    fn port_control(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::pctl::PORT_A,
            Port::F => registers::gpio::pctl::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the pull-down resistor select (PDR) register for this port
    fn pull_down_select(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::pdr::PORT_A,
            Port::F => registers::gpio::pdr::PORT_F,
            _ => todo!(),
        }
    }

    /// The memory address of the pull-up resistor select (PUR) register for this port
    fn pull_up_select(&self) -> *mut u32 {
        match self.port {
            Port::A => registers::gpio::pur::PORT_A,
            Port::F => registers::gpio::pur::PORT_F,
            _ => todo!(),
        }
    }

    // Note to self: page 1351 of data sheet for PWM
    // Apparently also for ADC!
}

impl PortIO {
    /// The corresponding bit for this port in system's run-mode clock gate control (RCGC) register
    fn run_mode_clock_gate_control(&self) -> Bit {
        match self.port {
            Port::A => Bit::Zero,
            Port::B => Bit::One,
            Port::C => Bit::Two,
            Port::D => Bit::Three,
            Port::E => Bit::Four,
            Port::F => Bit::Five,
        }
    }
}

impl PortIO {
    pub fn setup_readable_pins<const N: usize>(
        &self,
        bits: &[Bit; N],
        options: ReadablePinSetup,
    ) -> ReadablePins<N> {
        // Unlock the pins
        unsafe {
            memory::write(self.lock(), UNLOCK);

            memory::set_bits(self.commit(), &bits.map(|bit| bit as u32));
        }

        // Disable analog when it's not selected (and enable analog if it is)
        match options.function {
            Function::Analog => unsafe {
                memory::set_bits(self.analog_mode_select(), &bits.map(|bit| bit as u32));
            },
            _ => unsafe {
                memory::clear_bits(self.analog_mode_select(), &bits.map(|bit| bit as u32));
            },
        }

        unsafe {
            memory::clear_bits(self.direction(), &bits.map(|bit| bit as u32));
        }

        

        for bit in bits {
            let mut memory_bits = [0; 4];

            let min = (*bit as u32) * 4;
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
                memory::write_bits(self.port_control(), &memory_bits, values);
            }
        }


        // TODO: finish

        match options.pull_up {
            Some(true) => todo!(),
            Some(false) => todo!(),
            None => todo!(),
        }

        let data_address = self.data();

        let pins: [ReadablePin; N] = bits.map(|bit| ReadablePin { data_address, bit });

        ReadablePins { data_address, pins }
    }

    pub fn setup_writable_pins<const N: usize>(
        &self,
        bits: &[Bit; N],
        options: WritablePinSetup,
    ) -> WritablePins<N> {
        // Unlock the pins
        unsafe {
            memory::write(self.lock(), UNLOCK);

            memory::set_bits(self.commit(), &bits.map(|bit| bit as u32));
        }

        // Disable analog when it's not selected (and enable analog if it is)
        match options.function {
            Function::Analog => unsafe {
                memory::set_bits(self.analog_mode_select(), &bits.map(|bit| bit as u32));
            },
            _ => unsafe {
                memory::clear_bits(self.analog_mode_select(), &bits.map(|bit| bit as u32));
            },
        }

        unsafe {
            memory::set_bits(self.direction(), &bits.map(|bit| bit as u32));
        }

        for bit in bits {
            let mut memory_bits = [0; 4];

            let min = (*bit as u32) * 4;
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
                memory::write_bits(self.port_control(), &memory_bits, values);
            }
        }

        // TODO: check page 671 or 682 (+ more prob) for a table showing initial pin states

        // TODO: finish

        match options.function {
            Function::Analog | Function::Digital => {
                unsafe {
                    memory::clear_bits(self.alternate_function_select(), &bits.map(|bit| bit as u32));
                }
            },
            _ => {
                unsafe {
                    memory::set_bits(self.alternate_function_select(), &bits.map(|bit| bit as u32));
                }
            },
        }

        let data_address = self.data();

        let pins: [WritablePin; N] = bits.map(|bit| WritablePin { data_address, bit });

        WritablePins { data_address, pins }
    }
}

#[derive(Clone, Copy)]
pub enum Bit {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

/// Page 1351 of data sheet
pub enum Function {
    Analog,
    Digital,
    CAN,
    I2C,
    PWM,
    UART,
}

pub struct ReadablePinSetup {
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
    bit: Bit,
}
impl ReadablePin {
    pub fn read(&self) -> bool {
        let current = unsafe { memory::read(self.data_address) };
        current & (1 << self.bit as u32) != 0
    }
}

pub struct WritablePinSetup {
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
    bit: Bit,
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

pub fn setup_board() -> Board {
    Board
}

pub trait Rom {
    fn load(&self, address: u16) -> u8;
}

pub trait Ram: Rom {
    fn store(&mut self, address: u16, value: u8);
}

// Blanket implementation for anything that converts to a u8 slice
impl<T: AsRef<[u8]>> Rom for T {
    fn load(&self, address: u16) -> u8 {
        //TODO: Figure out behavior on out of bounds access
        match self.as_ref().get(address as usize) {
            Some(some) => *some,
            None => panic!("Invalid load address: {:#04X}", address),
        }
    }
}

// Blanket implementation for anything that converts to a mutable u8 slice
impl<T: Rom + AsMut<[u8]>> Ram for T {
    fn store(&mut self, address: u16, value: u8) {
        //TODO: Figure out behavior on out of bounds access
        match self.as_mut().get_mut(address as usize) {
            Some(some) => *some = value,
            None => panic!("Invalid store address: {:#04X}", address),
        }
    }
}

/// The basic 8051 Microcontroller
pub struct Mcu<P: Rom, X: Ram> {
    /// Program counter
    pub pc: u16,
    /// Program memory (PMEM)
    pub rom: P,
    /// Internal RAM (IRAM) and external data memory (XRAM)
    pub ram: X,
}

impl<P: Rom, X: Ram> Mcu<P, X> {
    pub fn new(pc: u16, rom: P, ram: X) -> Self {
        Self {
            pc,
            rom,
            ram
        }
    }

    pub fn a(&self) -> u8 {
        self.ram.load(0xE0)
    }

    pub fn set_a(&mut self, value: u8) {
        self.ram.store(0xE0, value);
    }

    pub fn dptr(&self) -> u16 {
        self.ram.load(0x82) as u16 | (self.ram.load(0x83) as u16) << 8
    }

    pub fn set_dptr(&mut self, value: u16) {
        self.ram.store(0x82, value as u8);
        self.ram.store(0x83, (value >> 8) as u8);
    }

    pub fn load_pc(&mut self) -> u8 {
        let value = self.rom.load(self.pc);
        self.pc += 1;
        value
    }

    pub fn step(&mut self) {
        let op = self.load_pc();
        match op {
            // nop
            0x00 => (),
            // mov dptr, #data16
            0x90 => {
                let value = (self.load_pc() as u16) << 8 | (self.load_pc() as u16);
                println!("  mov dptr, {:#02X}", value);
                self.set_dptr(value);
            },
            // mov a, #data
            0x74 => {
                let value = self.load_pc();
                println!("  mov a, #{:#02X}", value);
                self.set_a(value);
            },
            // mov a, iram addr
            0xE5 => {
                let address = self.load_pc();
                println!("  mov a, {:#02X}", address);
                let value = self.ram.load(address as u16);
                self.set_a(value);
            },
            // movx @dptr, a
            0xF0 => {
                println!("  movx @dptr, a");
                let address = self.dptr();
                let value = self.a();
                self.ram.store(address, value);
            },
            _ => panic!("Unknown opcode: {:#02X}", op),
        }
    }
}

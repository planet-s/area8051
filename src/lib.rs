pub trait Rom {
    fn load(&self, address: u16) -> u8;
}

pub trait Ram: Rom {
    fn store(&mut self, address: u16, value: u8);
}

// Blanket implementation for anything that converts to a u8 slice
impl<T> Rom for T where T: AsRef<[u8]> {
    fn load(&self, address: u16) -> u8 {
        //TODO: Figure out behavior on out of bounds access
        match self.as_ref().get(address as usize) {
            Some(some) => *some,
            None => panic!("Invalid load address: {:#>04x}", address),
        }
    }
}

// Blanket implementation for anything that converts to a mutable u8 slice
impl<T> Ram for T where T: Rom + AsMut<[u8]> {
    fn store(&mut self, address: u16, value: u8) {
        //TODO: Figure out behavior on out of bounds access
        match self.as_mut().get_mut(address as usize) {
            Some(some) => *some = value,
            None => panic!("Invalid store address: {:#>04x}", address),
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

    pub fn pc_load(&mut self) -> u8 {
        let value = self.rom.load(self.pc);
        self.pc += 1;
        value
    }

    pub fn step(&mut self) {
        let op = self.pc_load();
        println!("Opcode: {:#>02x}", op);
        match op {
            0 => (),
            _ => panic!("Unknown opcode: {:#>02x}", op),
        }
    }
}

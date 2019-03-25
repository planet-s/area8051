#[cfg(feature = "debug")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($($arg:tt)*) => (());
}

pub use self::addr::Addr;
mod addr;

pub use self::isa::Isa;
mod isa;

pub use self::mem::Mem;
mod mem;

pub use self::reg::Reg;
mod reg;

pub struct Mcu {
    pub pc: u16,
    pub iram: Box<[u8]>,
    pub sfr: Box<[u8]>,
    pub pmem: Box<[u8]>,
    pub xram: Box<[u8]>,
}

impl Mcu {
    pub fn new(pmem: Box<[u8]>) -> Self {
        Self {
            pc: 0,
            iram: vec![0; 256].into_boxed_slice(),
            sfr: vec![0; 128].into_boxed_slice(),
            pmem: pmem,
            xram: vec![0; 65536].into_boxed_slice(),
        }
    }
}

impl Mem for Mcu {
    fn load(&self, addr: Addr) -> u8 {
        match addr {
            Addr::Reg(i) => if i < 0x80 {
                self.iram[i as usize]
            } else {
                self.sfr[i as usize - 0x80]
            }
            Addr::IRam(i) => self.iram[i as usize],
            Addr::PMem(i) => self.pmem[i as usize],
            Addr::XRam(i) => self.xram[i as usize],
        }
    }

    fn store(&mut self, addr: Addr, value: u8) {
        match addr {
            Addr::Reg(i) => if i < 0x80 {
                self.iram[i as usize] = value
            } else {
                self.sfr[i as usize - 0x80] = value
            }
            Addr::IRam(i) => self.iram[i as usize] = value,
            Addr::PMem(_) => panic!("pmem cannot be written"),
            Addr::XRam(i) => self.xram[i as usize] = value,
        }
    }
}

impl Reg for Mcu {}

impl Isa for Mcu {
    fn pc(&self) -> u16 {
        self.pc
    }

    fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }
}

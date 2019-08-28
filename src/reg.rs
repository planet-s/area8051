use crate::{Addr, Mem};

pub trait Reg: Mem {
    fn r(&self, index: u8) -> Addr {
        if index >= 8 {
            panic!("Invalid register r{}", index);
        }
        let rs = (self.load(self.psw()) >> 3) & 3;
        Addr::Reg(rs * 8 + index)
    }

    fn bit(&self, bit: u8) -> (Addr, u8) {
        let byte = bit / 8;
        let addr = match byte {
            0 ..= 0xF => Addr::Reg(0x20 + byte),
            0x10 ..= 0x1F => Addr::Reg(byte * 8),
            _ => panic!("Invalid bit 0x{:02X}", bit),
        };
        let mask = 1 << (bit % 8);
        (addr, mask)
    }

    fn p(&self, index: u8) -> Addr {
        if index >= 4 {
            panic!("Invalid port p{}", index);
        }
        Addr::Reg(0x80 + index * 0x10)
    }

    fn sp(&self) -> Addr {
        Addr::Reg(0x81)
    }

    fn dptr(&self, index: bool) -> Addr {
        if self.load(self.dps()) & 1 == 0 {
            Addr::Reg(0x82 + (index as u8))
        } else {
            Addr::Reg(0x84 + (index as u8))
        }
    }

    fn dps(&self) -> Addr {
        Addr::Reg(0x86)
    }

    fn psw(&self) -> Addr {
        Addr::Reg(0xD0)
    }

    fn a(&self) -> Addr {
        Addr::Reg(0xE0)
    }

    fn b(&self) -> Addr {
        Addr::Reg(0xF0)
    }
}

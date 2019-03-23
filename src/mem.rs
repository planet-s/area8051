use crate::Addr;

pub trait Mem {
    fn load(&self, addr: Addr) -> u8;
    fn store(&mut self, addr: Addr, value: u8);
}

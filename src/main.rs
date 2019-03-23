use area8051::{Addr, Isa, Mcu, Mem};
use std::{env, fs};

fn main() {
    let file = env::args().nth(1).expect("rom file not provided");
    let pmem = fs::read(file).expect("failed to read rom file");

    let mut mcu = Mcu::new(pmem.into_boxed_slice());

    mcu.reset();

    loop {
        mcu.step();

        // Serial bus
        let s = mcu.load(Addr::Reg(0x98));
        let b = mcu.load(Addr::Reg(0x99));
        if b > 0 {
            print!("{}", b as char);
            mcu.store(Addr::Reg(0x98), s | (1 << 1));
            mcu.store(Addr::Reg(0x99), 0);
        }

        // Shutdown signal at 0xFFFF
        if mcu.load(Addr::XRam(0xFFFF)) > 0 {
            break;
        }
    }
}

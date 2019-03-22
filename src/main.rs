use area8051::{Mcu, Rom, Ram};
use std::{env, fs};

fn main() {
    let file = env::args().nth(1).expect("rom file not provided");
    let pmem = fs::read(file).expect("failed to read rom file");
    let iram = vec![0; 256];
    let xram = vec![0; 65536];

    let mut mcu = Mcu::new(0, pmem, iram, xram);

    mcu.reset();

    loop {
        mcu.step();

        // Serial bus
        let s = mcu.iram.load(0x98);
        let b = mcu.iram.load(0x99);
        if b > 0 {
            print!("{}", b as char);
            mcu.iram.store(0x98, s | (1 << 1));
            mcu.iram.store(0x99, 0);
        }

        // Shutdown signal at 0xFFFF
        if mcu.xram.load(0xFFFF) > 0 {
            break;
        }
    }
}

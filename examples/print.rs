use area8051::{Mcu, Rom, Ram};
use std::fs;

fn main() {
    let mut pmem = fs::read("examples/print.rom").expect("failed to read print.rom");
    let iram = vec![0; 256];
    let mut xram = pmem.clone();
    while xram.len() < 65536 {
        xram.push(0);
    }

    let mut mcu = Mcu::new(0, pmem, iram, xram);

    mcu.reset();

    loop {
        mcu.step();

        // Serial bus
        let b = mcu.iram.load(0x99);
        if b > 0 {
            println!("{}", b as char);
            mcu.xram.store(0x99, 0);
        }

        // Shutdown signal at 0xFFFF
        if mcu.xram.load(0xFFFF) > 0 {
            break;
        }
    }
}

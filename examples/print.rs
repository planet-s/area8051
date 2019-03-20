use area8051::{Mcu, Rom, Ram};
use std::fs;

fn main() {
    let rom = fs::read("examples/print.rom").expect("failed to read print.rom");
    let mut ram = vec![0u8; 65536];

    let mut mcu = Mcu::new(0, rom, ram);

    mcu.reset();

    loop {
        mcu.step();

        // We have a theoretical data bus at 0x400
        let b = mcu.ram.load(0x400);
        if b > 0 {
            println!("{}", b as char);
            mcu.ram.store(0x400, 0);
        }

        // Shutdown signal at 0xFFFF
        if mcu.ram.load(0xFFFF) > 0 {
            break;
        }
    }
}

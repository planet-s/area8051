use area8051::{Mcu, Ram, Rom};

fn main() {
    let rom = vec![0u8; 65536];
    let mut ram = vec![0u8; 65536];

    let mut mcu = Mcu::new(0, rom, ram);

    mcu.step();
}

#[cfg(feature = "debug")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($($arg:tt)*) => ();
}

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
            None => panic!("Invalid load address: {:#X}", address),
        }
    }
}

// Blanket implementation for anything that converts to a mutable u8 slice
impl<T: Rom + AsMut<[u8]>> Ram for T {
    fn store(&mut self, address: u16, value: u8) {
        //TODO: Figure out behavior on out of bounds access
        match self.as_mut().get_mut(address as usize) {
            Some(some) => *some = value,
            None => panic!("Invalid store address: {:#X}", address),
        }
    }
}

/// The basic 8051 Microcontroller
pub struct Mcu<P: Rom, I: Ram, X: Ram> {
    /// Program counter
    pub pc: u16,
    /// Program memory (PMEM)
    pub pmem: P,
    /// Internal RAM (IRAM) and
    pub iram: I,
    /// External data memory (XRAM)
    pub xram: X,
}

impl<P: Rom, I: Ram, X: Ram> Mcu<P, I, X> {
    pub fn new(pc: u16, pmem: P, iram: I, xram: X) -> Self {
        Self {
            pc,
            pmem,
            iram,
            xram
        }
    }

    /* Accumulator */
    pub fn a_addr(&self) -> u16 {
        0xE0
    }

    pub fn a(&self) -> u8 {
        self.iram.load(self.a_addr())
    }

    pub fn set_a(&mut self, value: u8) {
        self.iram.store(self.a_addr(), value);
    }

    /* Accumulator extension */
    pub fn b_addr(&self) -> u16 {
        0xF0
    }

    pub fn b(&self) -> u8 {
        self.iram.load(self.b_addr())
    }

    pub fn set_b(&mut self, value: u8) {
        self.iram.store(self.b_addr(), value);
    }

    /* Data pointer */
    pub fn dptr_addr(&self) -> u16 {
        if self.dps() & 1 == 0 {
            0x82
        } else {
            0x84
        }
    }

    pub fn dptr(&self) -> u16 {
        let addr = self.dptr_addr();
        self.iram.load(addr) as u16 |
        (self.iram.load(addr + 1) as u16) << 8
    }

    pub fn set_dptr(&mut self, value: u16) {
        let addr = self.dptr_addr();
        self.iram.store(addr, value as u8);
        self.iram.store(addr + 1, (value >> 8) as u8);
    }

    pub fn dps_addr(&self) -> u16 {
        0x86
    }

    pub fn dps(&self) -> u8 {
        self.iram.load(self.dps_addr())
    }

    pub fn set_dps(&mut self, value: u8) {
        self.iram.store(self.dps_addr(), value);
    }

    /* Program status word */
    pub fn psw_addr(&self) -> u16 {
        0xD0
    }

    pub fn psw(&self) -> u8 {
        self.iram.load(self.psw_addr())
    }

    pub fn set_psw(&mut self, value: u8) {
        self.iram.store(self.psw_addr(), value);
    }

    pub fn ov(&self) -> bool {
        self.psw() & (1 << 2) > 0
    }

    pub fn set_ov(&mut self, value: bool) {
        let mut psw = self.psw();
        psw &= !(1 << 2);
        psw |= (value as u8) << 2;
        self.set_psw(psw);
    }

    pub fn ac(&self) -> bool {
        self.psw() & (1 << 6) > 0
    }

    pub fn set_ac(&mut self, value: bool) {
        let mut psw = self.psw();
        psw &= !(1 << 6);
        psw |= (value as u8) << 6;
        self.set_psw(psw);
    }

    pub fn c(&self) -> bool {
        self.psw() & (1 << 7) > 0
    }

    pub fn set_c(&mut self, value: bool) {
        let mut psw = self.psw();
        psw &= !(1 << 7);
        psw |= (value as u8) << 7;
        self.set_psw(psw);
    }

    /* General registers */
    pub fn r_addr(&self, index: u8) -> u16 {
        if index >= 8 {
            panic!("Invalid register r{}", index);
        }
        let rs = (self.psw() >> 3) & 3;
        (rs * 8 + index) as u16
    }

    pub fn r(&self, index: u8) -> u8 {
        self.iram.load(self.r_addr(index))
    }

    pub fn set_r(&mut self, index: u8, value: u8) {
        self.iram.store(self.r_addr(index), value);
    }

    /* Stack pointer */
    pub fn sp_addr(&self) -> u16 {
        0x81
    }

    pub fn sp(&self) -> u8 {
        self.iram.load(self.sp_addr())
    }

    pub fn set_sp(&mut self, value: u8) {
        self.iram.store(self.sp_addr(), value)
    }

    /* Bit operations */
    pub fn bit(&self, bit: u8) -> (u16, u8) {
        let byte = bit / 8;
        let address = match byte {
            0 ... 0xF => (0x20 + byte) as u16,
            0x10 ... 0x1F => (byte * 8) as u16,
            _ => panic!("Invalid bit 0x{:02X}", bit),
        };
        let mask = 1 << (bit % 8);
        (address, mask)
    }

    /* Memory operations */
    pub fn pop_sp(&mut self) -> u8 {
        let sp = self.sp();
        let value = self.iram.load(sp as u16);
        self.set_sp(sp.wrapping_sub(1));
        value
    }

    pub fn push_sp(&mut self, value: u8) {
        let sp = self.sp().wrapping_add(1);
        self.set_sp(sp);
        self.iram.store(sp as u16, value);
    }

    pub fn load_pc(&mut self) -> u8 {
        let value = self.pmem.load(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    pub fn reljmp(&mut self, offset: u8) {
        self.pc = self.pc.wrapping_add((((offset as i8) as i16) as u16));
    }

    /* Processor operations */
    pub fn reset(&mut self) {
        self.pc = 0;

        // Clear all of lower memory
        for address in 0..256 {
            self.iram.store(address, 0);
        }

        // Clear general registers for all four banks, and clear program status word
        for &b in &[3, 2, 1, 0] {
            self.set_psw(b << 3);
            for r in 0..8 {
                self.set_r(r, 0);
            }
        }

        // Set accumulator and extension
        self.set_a(0);
        self.set_b(0);

        // Clear data pointers
        for &dps in &[1, 0] {
            self.set_dps(dps);
            self.set_dptr(0);
        }

        // Set stack pointer to default (7)
        self.set_sp(7);
    }

    pub fn operand(&mut self, op: u8) -> u16 {
        let operand = op & 0xF;
        match operand {
            0x4 => {
                debug!(" a");
                self.a_addr()
            },
            0x5 => {
                let value = self.load_pc() as u16;
                debug!(" 0x{:02X}", value);
                value
            },
            0x6 ... 0x7 => {
                let r = operand - 0x6;
                debug!(" @r{}", r);
                self.r(r) as u16
            },
            0x8 ... 0xF => {
                let r = operand - 0x8;
                debug!(" r{}", r);
                self.r_addr(r)
            },
            _ => panic!("Irregular operand: {:#X}", operand),
        }
    }

    pub fn step(&mut self) {
        debug!("  0x{:04X}: ", self.pc);

        let op = self.load_pc();
        match op {
            /* nop */
            0x00 => {
                debug!("nop");
            },

            /* ljmp address */
            0x02 => {
                let address = (self.load_pc() as u16) << 8 | (self.load_pc() as u16);
                debug!("ljmp 0x{:04X}", address);
                self.pc = address;
            },

            /* inc operand */
            0x04 ... 0x0F => {
                debug!("inc");
                let operand = self.operand(op);
                let value = self.iram.load(operand);
                self.iram.store(operand, value.wrapping_add(1));
            },

            /* jbs bit, offset */
            0x10 => {
                let bit = self.load_pc();
                let offset = self.load_pc();
                debug!("jbs 0x{:02X}, 0x{:02X}", bit, offset);
                let (address, mask) = self.bit(bit);
                let value = self.iram.load(address as u16);
                if value & mask != 0 {
                    self.iram.store(address as u16, value & !mask);
                    self.reljmp(offset);
                }
            },

            /* lcall address */
            0x12 => {
                let address = (self.load_pc() as u16) << 8 | (self.load_pc() as u16);
                debug!("lcall 0x{:04X}", address);
                self.push_sp(self.pc as u8);
                self.push_sp((self.pc >> 8) as u8);
                self.pc = address;
            },

            /* dec operand */
            0x14 ... 0x1F => {
                debug!("dec");
                let operand = self.operand(op);
                let value = self.iram.load(operand);
                self.iram.store(operand, value.wrapping_sub(1));
            },

            /* jb bit, offset */
            0x20 => {
                let bit = self.load_pc();
                let offset = self.load_pc();
                debug!("jb 0x{:02X}, 0x{:02X}", bit, offset);
                let (address, mask) = self.bit(bit);
                let value = self.iram.load(address as u16);
                if value & mask != 0 {
                    self.reljmp(offset);
                }
            },

            /* ret */
            0x22 => {
                debug!("ret");
                self.pc = (self.pop_sp() as u16) << 8 | (self.pop_sp() as u16);
            },

            /* add a, operand */
            0x24 ... 0x2F => {
                debug!("add a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.iram.load(operand)
                } as i16;

                let a = self.a() as i16;

                // Set carry if sum is greater than 0xFF
                self.set_c((value + a) > 0xFF);
                // Set auxiliary carry if low nibble sum is greater than 0xF
                self.set_ac(((value & 0xF) + (a & 0xF)) > 0xF);
                // Set overflow flag if signed result is not within range
                let signed = (value as i8) as i16 + (a as i8) as i16;
                self.set_ov(signed > 127 || signed < -128);

                self.set_a((a - value) as u8);
            },

            /* jnb bit, offset */
            0x30 => {
                let bit = self.load_pc();
                let offset = self.load_pc();
                debug!("jnb 0x{:02X}, 0x{:02X}", bit, offset);
                let (address, mask) = self.bit(bit);
                let value = self.iram.load(address as u16);
                if value & mask == 0 {
                    self.reljmp(offset);
                }
            },

            /* addc a, operand */
            0x34 ... 0x3F => {
                debug!("addc a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.iram.load(operand)
                } as i16 + self.c() as i16;

                let a = self.a() as i16;

                // Set carry if sum is greater than 0xFF
                self.set_c((value + a) > 0xFF);
                // Set auxiliary carry if low nibble sum is greater than 0xF
                self.set_ac(((value & 0xF) + (a & 0xF)) > 0xF);
                // Set overflow flag if signed result is not within range
                let signed = (value as i8) as i16 + (a as i8) as i16;
                self.set_ov(signed > 127 || signed < -128);

                self.set_a((a - value) as u8);
            },

            /* jc offset */
            0x40 => {
                let offset = self.load_pc();
                debug!("jc 0x{:02X}", offset);
                if self.c() {
                    self.reljmp(offset);
                }
            },

            /* orl address, #data */
            0x43 => {
                let address = self.load_pc();
                let value = self.load_pc();
                debug!("orl 0x{:02X}, #0x{:02X}", address, value);

                let iram = self.iram.load(address as u16);
                self.iram.store(address as u16, iram | value);
            }

            /* orl a, operand */
            0x44 ... 0x4F => {
                debug!("orl a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.iram.load(operand)
                };

                let a = self.a();
                self.set_a(a | value);
            },

            /* jnc offset */
            0x50 => {
                let offset = self.load_pc();
                debug!("jnc 0x{:02X}", offset);
                if ! self.c() {
                    self.reljmp(offset);
                }
            },

            /* anl address, #data */
            0x53 => {
                let address = self.load_pc();
                let value = self.load_pc();
                debug!("anl 0x{:02X}, #0x{:02X}", address, value);

                let iram = self.iram.load(address as u16);
                self.iram.store(address as u16, iram & value);
            }

            /* anl a, operand */
            0x54 ... 0x5F => {
                debug!("anl a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.iram.load(operand)
                };

                let a = self.a();
                self.set_a(a & value);
            },

            /* jz offset */
            0x60 => {
                let offset = self.load_pc();
                debug!("jz 0x{:02X}", offset);
                if self.a() == 0 {
                    self.reljmp(offset);
                }
            },

            /* xrl a, operand */
            0x64 ... 0x6F => {
                debug!("xrl a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.iram.load(operand)
                };

                let a = self.a();
                self.set_a(a ^ value);
            },

            /* jnz offset */
            0x70 => {
                let offset = self.load_pc();
                debug!("jnz 0x{:02X}", offset);
                if self.a() != 0 {
                    self.reljmp(offset);
                }
            },

            /* mov operand, #data */
            0x74 ... 0x7F => {
                debug!("mov");
                let operand = self.operand(op);
                let value = self.load_pc();
                debug!(", #0x{:02X}", value);
                self.iram.store(operand, value);
            },

            /* sjmp offset */
            0x80 => {
                let offset = self.load_pc();
                debug!("sjmp 0x{:02X}", offset);
                self.reljmp(offset);
            },

            /* mov address, address */
            0x85 => {
                let src = self.load_pc();
                let dest = self.load_pc();
                debug!("mov 0x{:02X}, 0x{:02X}", dest, src);
                let value = self.iram.load(src as u16);
                self.iram.store(dest as u16, value);
            },

            /* mov address, operand */
            0x86 ... 0x8F => {
                let address = self.load_pc();
                debug!("mov 0x{:02X},", address);
                let operand = self.operand(op);
                let value = self.iram.load(operand);
                self.iram.store(address as u16, value);
            },

            /* mov dptr, address */
            0x90 => {
                let address = (self.load_pc() as u16) << 8 | (self.load_pc() as u16);
                debug!("mov dptr, 0x{:04X}", address);
                self.set_dptr(address);
            },

            /* movc a, @a+dptr */
            0x93 => {
                debug!("movc a, @a+dptr");
                let address = self.dptr().wrapping_add(self.a() as u16);
                let value = self.pmem.load(address);
                self.set_a(value);
            },

            /* sub a, operand */
            0x94 ... 0x9F => {
                debug!("subb a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.iram.load(operand)
                } as i16 + self.c() as i16;

                let a = self.a() as i16;

                // Set carry if value being subtracted is greater than a
                self.set_c(value > a);
                // Set auxiliary carry if low nibble of value being subtraced is greater than low nibble of a
                self.set_ac((value & 0xF) > (a & 0xF));
                // Set overflow flag if signed result is not within range
                let signed = (a as i8) as i16 - (value as i8) as i16;
                self.set_ov(signed > 127 || signed < -128);

                self.set_a((a - value) as u8);
            },

            /* inc dptr */
            0xA3 => {
                debug!("inc dptr");
                let value = self.dptr();
                self.set_dptr(value.wrapping_add(1));
            },

            /* mov operand, address */
            0xA6 ... 0xAF => {
                debug!("mov");
                let operand = self.operand(op);
                let address = self.load_pc();
                debug!(", 0x{:02X}", address);
                let value = self.iram.load(address as u16);
                self.iram.store(operand, value);
            },

            /* cjne operand, #data, offset */
            0xB4 ... 0xBF => {
                debug!("cjne");
                let (a, b) = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" a, #0x{:02X},", value);
                    (self.a(), value)
                } else if (op & 0xF) == 5 {
                    let address = self.load_pc();
                    debug!(" a, 0x{:02X},", address);
                    (self.a(), self.iram.load(address as u16))
                } else {
                    let operand = self.operand(op);
                    let value = self.load_pc();
                    debug!(", #0x{:02X},", value);
                    (self.iram.load(operand), value)
                };

                let offset = self.load_pc();
                debug!(" 0x{:02X}", offset);
                self.set_c(a < b);
                if a != b {
                    self.reljmp(offset);
                }
            }

            /* push address */
            0xC0 => {
                let address = self.load_pc();
                debug!("push 0x{:02X}", address);
                let value = self.iram.load(address as u16);
                self.push_sp(value);
            },

            /* clr bit */
            0xC2 => {
                let bit = self.load_pc();
                debug!("clr 0x{:02X}", bit);
                let (address, mask) = self.bit(bit);
                let value = self.iram.load(address as u16);
                self.iram.store(address as u16, value & !mask);
            },

            /* clr c */
            0xC3 => {
                debug!("clr c");
                self.set_c(false);
            },

            /* pop address */
            0xD0 => {
                let address = self.load_pc();
                debug!("pop 0x{:02X}", address);
                let value = self.pop_sp();
                self.iram.store(address as u16, value);
            },

            /* setb bit */
            0xD2 => {
                let bit = self.load_pc();
                debug!("setb 0x{:02X}", bit);
                let (address, mask) = self.bit(bit);
                let value = self.iram.load(address as u16);
                self.iram.store(address as u16, value | mask);
            },

            /* djnz operand, offset */
            0xD5 | 0xD8 ... 0xDF => {
                debug!("djnz");
                let operand = self.operand(op);
                let offset = self.load_pc();
                debug!(", 0x{:02X}", offset);
                let value = self.iram.load(operand).wrapping_sub(1);
                self.iram.store(operand, value);
                if operand != 0 {
                    self.reljmp(offset);
                }
            },

            /* movx @dptr, a */
            0xE0 => {
                debug!("movx a, @dptr");
                let address = self.dptr();
                let value = self.xram.load(address);
                self.set_a(value);
            },

            /* movx @dptr, a */
            0xE2 ... 0xE3 => {
                let r = op - 0xE2;
                debug!("movx a, @r{}", r);
                let address = self.r(r);
                let value = self.xram.load(address as u16);
                self.set_a(value);
            },

            /* clr a */
            0xE4 => {
                debug!("clr a");
                self.set_a(0);
            }

            /* mov a, operand */
            0xE5 ... 0xEF => {
                debug!("mov a,");
                let operand = self.operand(op);
                let value = self.iram.load(operand);
                self.set_a(value);
            },

            /* movx @dptr, a */
            0xF0 => {
                debug!("movx @dptr, a");
                let address = self.dptr();
                let value = self.a();
                self.xram.store(address, value);
            },

            /* movx @r0, a */
            0xF2 ... 0xF3 => {
                let r = op - 0xF2;
                debug!("movx @r{}, a",r );
                let address = self.r(r);
                let value = self.a();
                self.xram.store(address as u16, value);
            },

            /* mov operand, a */
            0xF5 ... 0xFF => {
                debug!("mov");
                let operand = self.operand(op);
                debug!(", a");
                let value = self.a();
                self.iram.store(operand, value);
            },

            /* unknown opcode */
            _ => panic!("Unknown opcode: 0x{:02X}", op),
        }

        debug!("\n");
    }
}

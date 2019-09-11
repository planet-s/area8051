use crate::{Addr, Mem, Reg};

pub trait Isa: Mem + Reg {
    fn pc(&self) -> u16;

    fn set_pc(&mut self, value: u16);

    fn reljmp(&mut self, offset: i8) {
        let pc = self.pc().wrapping_add((offset as i16) as u16);
        self.set_pc(pc);
    }

    fn load_pc(&mut self) -> u8 {
        let value = self.load(Addr::PMem(self.pc()));
        self.reljmp(1);
        value
    }

    fn pop_sp(&mut self) -> u8 {
        let sp = self.load(self.sp());
        let value = self.load(Addr::IRam(sp));
        self.store(self.sp(), sp.wrapping_sub(1));
        value
    }

    fn push_sp(&mut self, value: u8) {
        let sp = self.load(self.sp()).wrapping_add(1);
        self.store(self.sp(), sp);
        self.store(Addr::IRam(sp), value);
    }

    fn update_psw(&mut self, carry: bool, aux_carry: bool, overflow: bool) {
        let mut psw = self.load(self.psw());

        if carry {
            psw |= 1 << 7;
        } else {
            psw &= !(1 << 7);
        }

        if aux_carry {
            psw |= 1 << 6;
        } else {
            psw &= !(1 << 6);
        }

        if overflow {
            psw |= 1 << 2;
        } else {
            psw &= !(1 << 2);
        }

        self.store(self.psw(), psw);
    }

    fn reset(&mut self) {
        self.set_pc(0);

        for i in 0x00..=0xFF {
            self.store(Addr::Reg(i), 0);
        }

        self.store(self.sp(), 7);
    }

    fn operand(&mut self, op: u8) -> Addr {
        let operand = op & 0xF;
        match operand {
            0x4 => {
                debug!(" a");
                self.a()
            },
            0x5 => {
                let value = self.load_pc();
                debug!(" 0x{:02X}", value);
                Addr::Reg(value)
            },
            0x6 ..= 0x7 => {
                let r = operand - 0x6;
                debug!(" @r{}", r);
                Addr::IRam(self.load(self.r(r)))
            },
            0x8 ..= 0xF => {
                let r = operand - 0x8;
                debug!(" r{}", r);
                self.r(r)
            },
            _ => panic!("Irregular operand: {:#X}", operand),
        }
    }

    fn step(&mut self) {
        debug!("  0x{:04X}: ", self.pc());

        let op = self.load_pc();
        match op {
            /* nop */
            0x00 => {
                debug!("nop");
            },

            /* ljmp address */
            0x02 => {
                let address = {
                    (self.load_pc() as u16) << 8 |
                    (self.load_pc() as u16)
                };
                debug!("ljmp 0x{:04X}", address);
                self.set_pc(address);
            },

            /* rr a */
            0x03 => {
                debug!("rr a");
                let old = self.load(self.a());
                let new = if old & 1 == 0 {
                    old >> 1
                } else {
                    (1 << 7) | (old >> 1)
                };
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* inc operand */
            0x04 ..= 0x0F => {
                debug!("inc");
                let operand = self.operand(op);
                let old = self.load(operand);
                let new = old.wrapping_add(1);
                self.store(operand, new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* jbs bit, offset */
            0x10 => {
                let bit = self.load_pc();
                let offset = self.load_pc();
                debug!("jbs 0x{:02X}, 0x{:02X}", bit, offset);
                let (address, mask) = self.bit(bit);
                let value = self.load(address);
                if value & mask != 0 {
                    //TODO: value debug
                    self.store(address, value & !mask);
                    self.reljmp(offset as i8);
                }
            },

            /* lcall address */
            0x12 => {
                let address = {
                    (self.load_pc() as u16) << 8 |
                    (self.load_pc() as u16)
                };
                debug!("lcall 0x{:04X}", address);
                let pc = self.pc();
                self.push_sp(pc as u8);
                self.push_sp((pc >> 8) as u8);
                self.set_pc(address);
            },

            /* rrc a */
            0x13 => {
                debug!("rrc a");
                let old = self.load(self.a());
                let psw = self.load(self.psw());
                if old & 1 == 0 {
                    self.store(self.psw(), psw & !(1 << 7));
                } else {
                    self.store(self.psw(), psw | (1 << 7));
                }
                let new = if psw & (1 << 7) == 0 {
                    old >> 1
                } else {
                    (1 << 7) | (old >> 1)
                };
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* dec operand */
            0x14 ..= 0x1F => {
                debug!("dec");
                let operand = self.operand(op);
                let old = self.load(operand);
                let new = old.wrapping_sub(1);
                self.store(operand, new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* jb bit, offset */
            0x20 => {
                let bit = self.load_pc();
                let offset = self.load_pc();
                debug!("jb 0x{:02X}, 0x{:02X}", bit, offset);
                let (address, mask) = self.bit(bit);
                let value = self.load(address);
                if value & mask != 0 {
                    //TODO: value debug
                    self.reljmp(offset as i8);
                }
            },

            /* ret */
            0x22 => {
                debug!("ret");
                let pc = {
                    (self.pop_sp() as u16) << 8 |
                    (self.pop_sp() as u16)
                };
                self.set_pc(pc);
            },

            /* rl a */
            0x23 => {
                debug!("rl a");
                let old = self.load(self.a());
                let new = if old & (1 << 7) == 0 {
                    old << 1
                } else {
                    (old << 1) | 1
                };
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* add a, operand */
            0x24 ..= 0x2F => {
                debug!("add a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.load(operand)
                } as i16;

                let old = self.load(self.a()) as i16;

                // Set carry if sum is greater than 0xFF
                let carry = (value + old) > 0xFF;
                // Set auxiliary carry if low nibble sum is greater than 0xF
                let aux_carry = ((value & 0xF) + (old & 0xF)) > 0xF;
                // Set overflow flag if signed result is not within range
                let signed = (value as i8) as i16 + (old as i8) as i16;
                let overflow =  signed > 127 || signed < -128;
                self.update_psw(carry, aux_carry, overflow);

                let new = (old + value) as u8;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} += 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* jnb bit, offset */
            0x30 => {
                let bit = self.load_pc();
                let offset = self.load_pc();
                debug!("jnb 0x{:02X}, 0x{:02X}", bit, offset);
                let (address, mask) = self.bit(bit);
                let value = self.load(address);
                if value & mask == 0 {
                    //TODO: value debug
                    self.reljmp(offset as i8);
                }
            },

            /* rlc a */
            0x33 => {
                debug!("rlc a");
                let old = self.load(self.a());
                let psw = self.load(self.psw());
                if old & (1 << 7) == 0 {
                    self.store(self.psw(), psw & !(1 << 7));
                } else {
                    self.store(self.psw(), psw | (1 << 7));
                }
                let new = if psw & (1 << 7) == 0 {
                    old << 1
                } else {
                    (old << 1) | 1
                };
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* addc a, operand */
            0x34 ..= 0x3F => {
                debug!("addc a,");
                let mut value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.load(operand)
                } as i16;

                if self.load(self.psw()) & (1 << 7) != 0 {
                    value += 1;
                }

                let old = self.load(self.a()) as i16;

                // Set carry if sum is greater than 0xFF
                let carry = (value + old) > 0xFF;
                // Set auxiliary carry if low nibble sum is greater than 0xF
                let aux_carry = ((value & 0xF) + (old & 0xF)) > 0xF;
                // Set overflow flag if signed result is not within range
                let signed = (value as i8) as i16 + (old as i8) as i16;
                let overflow = signed > 127 || signed < -128;
                self.update_psw(carry, aux_carry, overflow);

                let new = (old + value) as u8;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} += 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* jc offset */
            0x40 => {
                let offset = self.load_pc();
                debug!("jc 0x{:02X}", offset);
                if self.load(self.psw()) & (1 << 7) != 0 {
                    //TODO: value debug
                    self.reljmp(offset as i8);
                }
            },

            /* orl address, a */
            0x42 => {
                let address = self.load_pc();
                debug!("orl 0x{:02X}, a", address);

                let value = self.load(self.a());
                let old = self.load(Addr::Reg(address));
                let new = old | value;
                self.store(Addr::Reg(address), new);
                debug!(" ; 0x{:02X} |= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* orl address, #data */
            0x43 => {
                let address = self.load_pc();
                let value = self.load_pc();
                debug!("orl 0x{:02X}, #0x{:02X}", address, value);

                let old = self.load(Addr::Reg(address));
                let new = old | value;
                self.store(Addr::Reg(address), new);
                debug!(" ; 0x{:02X} |= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* orl a, operand */
            0x44 ..= 0x4F => {
                debug!("orl a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.load(operand)
                };

                let old = self.load(self.a());
                let new = old | value;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} |= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* jnc offset */
            0x50 => {
                let offset = self.load_pc();
                debug!("jnc 0x{:02X}", offset);
                if self.load(self.psw()) & (1 << 7) == 0 {
                    //TODO: value debug
                    self.reljmp(offset as i8);
                }
            },

            /* anl address, a */
            0x52 => {
                let address = self.load_pc();
                debug!("anl 0x{:02X}, a", address);

                let value = self.load(self.a());
                let old = self.load(Addr::Reg(address));
                let new = old & value;
                self.store(Addr::Reg(address), new);
                debug!(" ; 0x{:02X} &= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* anl address, #data */
            0x53 => {
                let address = self.load_pc();
                let value = self.load_pc();
                debug!("anl 0x{:02X}, #0x{:02X}", address, value);

                let old = self.load(Addr::Reg(address));
                let new = old & value;
                self.store(Addr::Reg(address), new);
                debug!(" ; 0x{:02X} &= 0x{:02X} => 0x{:02X}", old, value, new);
            }

            /* anl a, operand */
            0x54 ..= 0x5F => {
                debug!("anl a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.load(operand)
                };

                let old = self.load(self.a());
                let new = old & value;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} &= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* jz offset */
            0x60 => {
                let offset = self.load_pc();
                debug!("jz 0x{:02X}", offset);
                if self.load(self.a()) == 0 {
                    //TODO: value debug
                    self.reljmp(offset as i8);
                }
            },

            /* xrl a, operand */
            0x64 ..= 0x6F => {
                debug!("xrl a,");
                let value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.load(operand)
                };

                let old = self.load(self.a());
                let new = old ^ value;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} ^= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* jnz offset */
            0x70 => {
                let offset = self.load_pc();
                debug!("jnz 0x{:02X}", offset);
                if self.load(self.a()) != 0 {
                    //TODO: value debug
                    self.reljmp(offset as i8);
                }
            },

            /* jmp @a+dptr */
            0x73 => {
                debug!("jmp @a+dptr");
                let address = (
                    (self.load(self.dptr(false)) as u16) |
                    (self.load(self.dptr(true)) as u16) << 8
                ).wrapping_add(self.load(self.a()) as u16);
                self.set_pc(address);
            },

            /* mov operand, #data */
            0x74 ..= 0x7F => {
                debug!("mov");
                let operand = self.operand(op);
                let value = self.load_pc();
                debug!(", #0x{:02X}", value);
                self.store(operand, value);
            },

            /* sjmp offset */
            0x80 => {
                let offset = self.load_pc();
                debug!("sjmp 0x{:02X}", offset);
                self.reljmp(offset as i8);
            },

            /* mov address, address */
            0x85 => {
                let src = self.load_pc();
                let dest = self.load_pc();
                debug!("mov 0x{:02X}, 0x{:02X}", dest, src);
                let value = self.load(Addr::Reg(src));
                self.store(Addr::Reg(dest), value);
                debug!(" ; 0x{:02X}", value);
            },

            /* mov address, operand */
            0x86 ..= 0x8F => {
                let address = self.load_pc();
                debug!("mov 0x{:02X},", address);
                let operand = self.operand(op);
                let value = self.load(operand);
                self.store(Addr::Reg(address), value);
                debug!(" ; 0x{:02X}", value);
            },

            /* mov dptr, address */
            0x90 => {
                let value = (self.load_pc() as u16) << 8 | (self.load_pc() as u16);
                debug!("mov dptr, 0x{:04X}", value);
                self.store(self.dptr(false), value as u8);
                self.store(self.dptr(true), (value >> 8) as u8);
            },

            /* mov bit, C */
            0x92 => {
                let bit = self.load_pc();
                debug!("mov 0x{:02X}, c", bit);
                let (address, mask) = self.bit(bit);
                let old = self.load(address);
                let new = if self.load(self.psw()) & (1 << 7) == 0 {
                    old & !mask
                } else {
                    old | mask
                };
                self.store(address, new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* movc a, @a+dptr */
            0x93 => {
                debug!("movc a, @a+dptr");
                let address = (
                    (self.load(self.dptr(false)) as u16) |
                    (self.load(self.dptr(true)) as u16) << 8
                ).wrapping_add(self.load(self.a()) as u16);
                let value = self.load(Addr::PMem(address));
                self.store(self.a(), value);
                debug!(" ; 0x{:04X}: 0x{:02X}", address, value);
            },

            /* sub a, operand */
            0x94 ..= 0x9F => {
                debug!("subb a,");
                let mut value = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" #0x{:02X}", value);
                    value
                } else {
                    let operand = self.operand(op);
                    self.load(operand)
                } as i16;

                if self.load(self.psw()) & (1 << 7) != 0 {
                    value += 1;
                }

                let old = self.load(self.a()) as i16;

                // Set carry if value being subtracted is greater than a
                let carry = value > old;
                // Set auxiliary carry if low nibble of value being subtraced is greater than low nibble of a
                let aux_carry = (value & 0xF) > (old & 0xF);
                // Set overflow flag if signed result is not within range
                let signed = (old as i8) as i16 - (value as i8) as i16;
                let overflow = signed > 127 || signed < -128;
                self.update_psw(carry, aux_carry, overflow);

                let new = (old - value) as u8;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} -= 0x{:02X} => 0x{:02X}", old, value, new);
            },

            /* inc dptr */
            0xA3 => {
                debug!("inc dptr");
                let value = (
                    (self.load(self.dptr(false)) as u16) |
                    (self.load(self.dptr(true)) as u16) << 8
                ).wrapping_add(1);
                self.store(self.dptr(false), value as u8);
                self.store(self.dptr(true), (value >> 8) as u8);
                debug!(" ; 0x{:04X}", value);
            },

            /* mul ab */
            0xA4 => {
                debug!("mul ab");
                let a = self.load(self.a());
                let b = self.load(self.b());

                let value = (a as u16) * (b as u16);
                self.update_psw(false, false, value > 255);

                self.store(self.a(), value as u8);
                self.store(self.b(), (value >> 8) as u8);
                debug!(" ; 0x{:02X} * 0x{:02X} => 0x{:04X}", a, b, value);
            },

            /* mov operand, address */
            0xA6 ..= 0xAF => {
                debug!("mov");
                let operand = self.operand(op);
                let address = self.load_pc();
                debug!(", 0x{:02X}", address);
                let value = self.load(Addr::Reg(address));
                self.store(operand, value);
                debug!(" ; 0x{:02X}", value);
            },

            /* cjne operand, #data, offset */
            0xB4 ..= 0xBF => {
                debug!("cjne");
                let (a, b) = if (op & 0xF) == 4 {
                    let value = self.load_pc();
                    debug!(" a, #0x{:02X},", value);
                    (self.load(self.a()), value)
                } else if (op & 0xF) == 5 {
                    let address = self.load_pc();
                    debug!(" a, 0x{:02X},", address);
                    (self.load(self.a()), self.load(Addr::Reg(address)))
                } else {
                    let operand = self.operand(op);
                    let value = self.load_pc();
                    debug!(", #0x{:02X},", value);
                    (self.load(operand), value)
                };

                let offset = self.load_pc();
                debug!(" 0x{:02X}", offset);
                let mut psw = self.load(self.psw());
                if a < b {
                    psw |= 1 << 7;
                } else {
                    psw &= !(1 << 7);
                }
                self.store(self.psw(), psw);
                //TODO: value debug
                if a != b {
                    self.reljmp(offset as i8);
                }
            },

            /* push address */
            0xC0 => {
                let address = self.load_pc();
                debug!("push 0x{:02X}", address);
                let value = self.load(Addr::Reg(address));
                self.push_sp(value);
                debug!(" ; 0x{:02X}", value);
            },

            /* clr bit */
            0xC2 => {
                let bit = self.load_pc();
                debug!("clr 0x{:02X}", bit);
                let (address, mask) = self.bit(bit);
                let old = self.load(address);
                let new = old & !mask;
                self.store(address, new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* clr c */
            0xC3 => {
                debug!("clr c");
                let psw = self.load(self.psw());
                self.store(self.psw(), psw & !(1 << 7));
            },

            /* swap a */
            0xC4 => {
                debug!("swap a");
                let old = self.load(self.a());
                let new = (old >> 4) | (old & 0xF) << 4;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* xch a, operand */
            0xC5 ..= 0xCF => {
                debug!("xch a,");
                let operand = self.operand(op);
                let old = self.load(self.a());
                let new = self.load(operand);
                self.store(self.a(), new);
                self.store(operand, old);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* pop address */
            0xD0 => {
                let address = self.load_pc();
                debug!("pop 0x{:02X}", address);
                let value = self.pop_sp();
                self.store(Addr::Reg(address), value);
                debug!(" ; 0x{:02X}", value);
            },

            /* setb bit */
            0xD2 => {
                let bit = self.load_pc();
                debug!("setb 0x{:02X}", bit);
                let (address, mask) = self.bit(bit);
                let old = self.load(address);
                let new = old | mask;
                self.store(address, new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* setb c */
            0xD3 => {
                debug!("setb c");
                let value = self.load(self.psw());
                self.store(self.psw(), value | (1 << 7));
            },

            /* djnz operand, offset */
            0xD5 | 0xD8 ..= 0xDF => {
                debug!("djnz");
                let operand = self.operand(op);
                let offset = self.load_pc();
                debug!(", 0x{:02X}", offset);
                let value = self.load(operand).wrapping_sub(1);
                self.store(operand, value);
                //TODO: value debug
                if value != 0 {
                    self.reljmp(offset as i8);
                }
            },

            /* movx a, @dptr */
            0xE0 => {
                debug!("movx a, @dptr");
                let address = {
                    (self.load(self.dptr(false)) as u16) |
                    (self.load(self.dptr(true)) as u16) << 8
                };
                let value = self.load(Addr::XRam(address));
                self.store(self.a(), value);
                debug!(" ; 0x{:04X}: 0x{:02X}", address, value);
            },

            /* movx a, rX */
            0xE2 ..= 0xE3 => {
                let r = op - 0xE2;
                debug!("movx a, @r{}", r);
                let address = {
                    (self.load(self.r(r)) as u16) |
                    (self.load(self.p(2)) as u16) << 8
                };
                let value = self.load(Addr::XRam(address));
                self.store(self.a(), value);
                debug!(" ; 0x{:04X}: 0x{:02X}", address, value);
            },

            /* clr a */
            0xE4 => {
                debug!("clr a");
                self.store(self.a(), 0);
            }

            /* mov a, operand */
            0xE5 ..= 0xEF => {
                debug!("mov a,");
                let operand = self.operand(op);
                let value = self.load(operand);
                self.store(self.a(), value);
                debug!(" ; 0x{:02X}", value);
            },

            /* movx @dptr, a */
            0xF0 => {
                debug!("movx @dptr, a");
                let address = {
                    (self.load(self.dptr(false)) as u16) |
                    (self.load(self.dptr(true)) as u16) << 8
                };
                let value = self.load(self.a());
                self.store(Addr::XRam(address), value);
                debug!(" ; 0x{:04X}: 0x{:02X}", address, value);
            },

            /* movx @r0, a */
            0xF2 ..= 0xF3 => {
                let r = op - 0xF2;
                debug!("movx @r{}, a",r );
                let address = {
                    (self.load(self.r(r)) as u16) |
                    (self.load(self.p(2)) as u16) << 8
                };
                let value = self.load(self.a());
                self.store(Addr::XRam(address), value);
                debug!(" ; 0x{:04X}: 0x{:02X}", address, value);
            },

            /* cpl a */
            0xF4 => {
                debug!("cpl a");
                let old = self.load(self.a());
                let new = !old;
                self.store(self.a(), new);
                debug!(" ; 0x{:02X} => 0x{:02X}", old, new);
            },

            /* mov operand, a */
            0xF5 ..= 0xFF => {
                debug!("mov");
                let operand = self.operand(op);
                debug!(", a");
                let value = self.load(self.a());
                self.store(operand, value);
                debug!(" ; 0x{:02X}", value);
            },

            /* unknown opcode */
            _ => panic!("Unknown opcode: 0x{:02X}", op),
        }

        debug!("\n");
    }
}

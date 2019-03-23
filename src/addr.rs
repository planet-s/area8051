#[derive(Clone, Copy)]
pub enum Addr {
    Reg(u8),
    IRam(u8),
    PMem(u16),
    XRam(u16),
}

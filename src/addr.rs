#[derive(Clone, Copy)]
pub enum Addr {
    /// Registers
    /// 256 bytes, accessed with direct access
    /// Low 128 bytes are shared with internal ram, high 128 bytes are special function registers
    Reg(u8),
    /// Internal RAM
    /// 256 bytes, accessed with indirect access
    /// Low 128 bytes are also accessible with direct access
    IRam(u8),
    /// Program memory
    /// 65536 bytes, accessed with movc, read-only
    PMem(u16),
    /// External RAM
    /// 65536 bytes, accessed with movx
    XRam(u16),
}

/// this trait is implemented by all SOFT packet types
pub trait SoftPacket {
    fn version(&self) -> u8;
    fn packet_type(&self) -> u8;
}
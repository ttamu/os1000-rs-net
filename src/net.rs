pub mod checksum;
pub mod ip;
pub mod loopback;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
    NoRoute,
    InvalidPacket,
    Timeout,
}

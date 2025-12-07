pub mod checksum;
pub mod ip;

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
    NoRoute,
    InvalidPacket,
    Timeout,
}

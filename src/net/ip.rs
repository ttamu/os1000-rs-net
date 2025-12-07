use core::fmt;

#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpV4Addr([u8; 4]);
impl IpV4Addr{
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    pub const LOOPBACK: IpV4Addr = IpV4Addr::new(127, 0, 0, 1);
    pub const ANY: IpV4Addr = IpV4Addr::new(0, 0, 0, 0);

    pub fn bytes(&self) -> [u8; 4] {
        self.0
    }
    pub fn is_loopback(&self) -> bool {
        self.0[0] == 127 // 127.0.0.0/8
    }
    pub fn to_be_u32(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }
    pub fn from_be_u32(n: u32) -> Self {
        Self(n.to_be_bytes())
    }
}

impl fmt::Display for IpV4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl fmt::Debug for IpV4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}
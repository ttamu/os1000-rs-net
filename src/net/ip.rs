use super::checksum::InternetChecksum;
use super::loopback::LoopbackInterface;
use core::fmt;
use core::mem::size_of;

#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpV4Addr([u8; 4]);
impl IpV4Addr {
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

#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct IpV4Protocol(u8);
impl IpV4Protocol {
    pub const fn icmp() -> Self {
        Self(1)
    }
    pub const fn tcp() -> Self {
        Self(6)
    }
    pub const fn udp() -> Self {
        Self(17)
    }
}

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct IpV4Header {
    version_and_ihl: u8,        // version (4bit) + IHL (4bit)
    dscp_and_ecn: u8,           // DSCP + ECN
    total_length: [u8; 2],      // 全長 (BE)
    identification: u16,        // 識別子
    flags_and_offset: u16,      // フラグ + フラグメントオフセット
    ttl: u8,                    // 生存時間
    protocol: IpV4Protocol,     // プロトコル
    checksum: InternetChecksum, // ヘッダチェックサム
    src_addr: IpV4Addr,         // 送信元アドレス
    dst_addr: IpV4Addr,         // 宛先アドレス
}
impl IpV4Header {
    pub fn new(src: IpV4Addr, dst: IpV4Addr, protocol: IpV4Protocol, data_len: usize) -> Self {
        let total_len = size_of::<Self>() + data_len;
        let mut header = Self {
            version_and_ihl: 0x45,
            dscp_and_ecn: 0,
            total_length: (total_len as u16).to_be_bytes(),
            identification: 0,
            flags_and_offset: 0,
            ttl: 64,
            protocol,
            checksum: InternetChecksum::default(),
            src_addr: src,
            dst_addr: dst,
        };

        let header_bytes = unsafe {
            core::slice::from_raw_parts(&header as *const _ as *const u8, size_of::<Self>())
        };
        header.checksum = InternetChecksum::calc(header_bytes);

        header
    }
    pub fn src(&self) -> IpV4Addr {
        self.src_addr
    }
    pub fn dst(&self) -> IpV4Addr {
        self.dst_addr
    }
    pub fn protocol(&self) -> IpV4Protocol {
        self.protocol
    }
}

static mut LOOPBACK: Option<LoopbackInterface> = None;
pub fn init() {
    unsafe {
        LOOPBACK = Some(LoopbackInterface::new());
    }
    crate::println!("[net] IP layer initialized");
}

pub fn ip_send(dst: IpV4Addr, protocol: IpV4Protocol, data: &[u8]) -> Result<(), super::NetError> {
    if !dst.is_loopback() {
        return Err(super::NetError::NoRoute);
    }

    let src = IpV4Addr::LOOPBACK;
    let header = IpV4Header::new(src, dst, protocol, data.len());

    const MAX_IP_PACKET: usize = 1500;
    let mut packet = [0u8; MAX_IP_PACKET];
    let header_size = size_of::<IpV4Header>();

    if header_size + data.len() > MAX_IP_PACKET {
        return Err(super::NetError::InvalidPacket);
    }

    unsafe {
        core::ptr::copy_nonoverlapping(
            &header as *const _ as *const u8,
            packet.as_mut_ptr(),
            header_size,
        );
    }
    packet[header_size..header_size + data.len()].copy_from_slice(data);

    unsafe {
        LOOPBACK
            .as_mut()
            .unwrap()
            .send(&packet[..header_size + data.len()])
    }
}

pub fn process_packets() {
    unsafe {
        let loopback = match LOOPBACK.as_mut() {
            Some(lo) => lo,
            None => return,
        };

        loop {
            let packet = match loopback.recv() {
                Some(p) => p,
                None => return,
            };

            if packet.len() < size_of::<IpV4Header>() {
                loopback.consume();
                continue;
            }

            let header = core::ptr::read_unaligned(packet.as_ptr() as *const IpV4Header);
            let data = &packet[size_of::<IpV4Header>()..];

            if header.protocol == IpV4Protocol::icmp() {
                super::icmp::handle_icmp_packet(header.src_addr, data);
            }

            loopback.consume();
        }
    }
}

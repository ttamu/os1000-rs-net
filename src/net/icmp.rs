use super::checksum::InternetChecksum;
use super::ip::{ip_send, IpV4Addr, IpV4Protocol};
use core::mem::size_of;

#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct IcmpType(u8);
impl IcmpType {
    pub fn echo_reply() -> Self {
        Self(0)
    }
    pub fn echo_request() -> Self {
        Self(8)
    }
}

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct IcmpEchoHeader {
    icmp_type: IcmpType,
    code: u8,
    checksum: InternetChecksum,
    identifier: u16,
    sequence: u16,
}
impl IcmpEchoHeader {
    pub fn new_request(id: u16, seq: u16) -> Self {
        Self {
            icmp_type: IcmpType::echo_request(),
            code: 0,
            checksum: InternetChecksum::default(),
            identifier: id.to_be(),
            sequence: seq.to_be(),
        }
    }

    pub fn new_reply(id: u16, seq: u16) -> Self {
        Self {
            icmp_type: IcmpType::echo_reply(),
            code: 0,
            checksum: InternetChecksum::default(),
            identifier: id.to_be(),
            sequence: seq.to_be(),
        }
    }
}
pub fn send_echo_request(
    dst: IpV4Addr,
    id: u16,
    seq: u16,
    data: &[u8],
) -> Result<(), super::NetError> {
    const MAX_ICMP_PACKET: usize = 1480; // IP MTU 1500 - IPv4ヘッダ20
    let mut packet = [0u8; MAX_ICMP_PACKET];
    let header_size = size_of::<IcmpEchoHeader>();

    if header_size + data.len() > MAX_ICMP_PACKET {
        return Err(super::NetError::InvalidPacket);
    }

    let mut header = IcmpEchoHeader::new_request(id, seq);

    unsafe {
        core::ptr::copy_nonoverlapping(
            &header as *const _ as *const u8,
            packet.as_mut_ptr(),
            header_size,
        );
    }
    packet[header_size..header_size + data.len()].copy_from_slice(data);

    let checksum = InternetChecksum::calc(&packet[..header_size + data.len()]);
    header.checksum = checksum;

    unsafe {
        core::ptr::copy_nonoverlapping(
            &header as *const _ as *const u8,
            packet.as_mut_ptr(),
            header_size,
        );
    }

    ip_send(
        dst,
        IpV4Protocol::icmp(),
        &packet[..header_size + data.len()],
    )
}

pub fn send_echo_reply(
    dst: IpV4Addr,
    id: u16,
    seq: u16,
    data: &[u8],
) -> Result<(), super::NetError> {
    const MAX_ICMP_PACKET: usize = 1480;
    let mut packet = [0u8; MAX_ICMP_PACKET];
    let header_size = size_of::<IcmpEchoHeader>();

    if header_size + data.len() > MAX_ICMP_PACKET {
        return Err(super::NetError::InvalidPacket);
    }

    let mut header = IcmpEchoHeader::new_reply(id, seq);

    unsafe {
        core::ptr::copy_nonoverlapping(
            &header as *const _ as *const u8,
            packet.as_mut_ptr(),
            header_size,
        );
    }
    packet[header_size..header_size + data.len()].copy_from_slice(data);

    let checksum = InternetChecksum::calc(&packet[..header_size + data.len()]);
    header.checksum = checksum;

    unsafe {
        core::ptr::copy_nonoverlapping(
            &header as *const _ as *const u8,
            packet.as_mut_ptr(),
            header_size,
        );
    }

    ip_send(
        dst,
        IpV4Protocol::icmp(),
        &packet[..header_size + data.len()],
    )
}

pub fn handle_icmp_packet(src: IpV4Addr, data: &[u8]) {
    if data.len() < size_of::<IcmpEchoHeader>() {
        return;
    }

    let header = unsafe { core::ptr::read_unaligned(data.as_ptr() as *const IcmpEchoHeader) };

    match header.icmp_type {
        t if t == IcmpType::echo_request() => {
            let id = u16::from_be(header.identifier);
            let seq = u16::from_be(header.sequence);
            let payload = &data[size_of::<IcmpEchoHeader>()..];

            crate::println!(
                "[icmp] Received Echo Request from {}, id={}, seq={}",
                src,
                id,
                seq
            );

            if let Err(e) = send_echo_reply(src, id, seq, payload) {
                crate::println!("[icmp] Failed to send Echo Reply: {:?}", e);
            }
        }
        t if t == IcmpType::echo_reply() => {
            let id = u16::from_be(header.identifier);
            let seq = u16::from_be(header.sequence);

            crate::println!(
                "[icmp] Received Echo Reply from {}, id={}, seq={}",
                src,
                id,
                seq
            );

            // TODO: ユーザープロセスに通知
        }
        _ => {
            crate::println!("[icmp] Unknown ICMP type: {:?}", header.icmp_type);
        }
    }
}

pub fn init() {
    crate::println!("[net] ICMP layer initialized");
}

// Loopback インターフェース実装
// loopback addressへのパケット送受信を処理

// TODO: グローバルアロケータ実装後、VecDeque<Vec<u8>>に変更する
// 現在は固定サイズ配列でリングバッファを実装

// TODO: NetDeviceトレイトで抽象化する
// virtio-net実装時に、loopback/virtio-net/その他のデバイスを統一的なインターフェースで扱えるようにリファクタリングする

const MAX_PACKETS: usize = 16;
const MAX_PACKET_SIZE: usize = 1514; // Ethernet MTU 1500 + Ethernetヘッダ14

pub struct LoopbackInterface {
    packets: [Option<[u8; MAX_PACKET_SIZE]>; MAX_PACKETS],
    packet_lens: [usize; MAX_PACKETS],
    head: usize,
    tail: usize,
    count: usize,
}
impl LoopbackInterface {
    pub fn new() -> Self {
        Self {
            packets: [None; MAX_PACKETS],
            packet_lens: [0; MAX_PACKETS],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn send(&mut self, packet: &[u8]) -> Result<(), super::NetError> {
        if self.count >= MAX_PACKETS {
            return Err(super::NetError::Timeout);
        }

        if packet.len() > MAX_PACKET_SIZE {
            return Err(super::NetError::InvalidPacket);
        }

        let mut buf = [0u8; MAX_PACKET_SIZE];
        buf[..packet.len()].copy_from_slice(packet);

        self.packets[self.tail] = Some(buf);
        self.packet_lens[self.tail] = packet.len();
        self.tail = (self.tail + 1) % MAX_PACKETS;
        self.count += 1;

        Ok(())
    }

    pub fn recv(&mut self) -> Option<&[u8]> {
        if self.count == 0 {
            return None;
        }

        let packet = self.packets[self.head].as_ref()?;
        let len = self.packet_lens[self.head];

        Some(&packet[..len])
    }

    pub fn consume(&mut self) {
        if self.count > 0 {
            self.packets[self.head] = None;
            self.head = (self.head + 1) % MAX_PACKETS;
            self.count -= 1;
        }
    }
}

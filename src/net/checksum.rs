#[repr(packed)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct InternetChecksum([u8; 2]);
impl InternetChecksum {
    pub fn calc(data: &[u8]) -> Self {
        InternetChecksumGenerator::new().feed(data).checksum()
    }
}

#[derive(Copy, Clone, Default)]
pub struct InternetChecksumGenerator {
    sum: u32,
}
impl InternetChecksumGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, data: &[u8]) -> &mut Self {
        let iter = data.chunks(2);
        for w in iter {
            self.sum += ((w[0] as u32) << 8) | w.get(1).cloned().unwrap_or_default() as u32;
        }
        self
    }

    pub fn checksum(&mut self) -> InternetChecksum {
        while (self.sum >> 16) != 0 {
            self.sum = (self.sum & 0xffff) + (self.sum >> 16);
        }
        InternetChecksum((!self.sum as u16).to_be_bytes())
    }
}

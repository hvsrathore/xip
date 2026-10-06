pub struct IndexHeader {
    sign: [u8; 4],
    version: u32,
    pub count: u32
}

impl IndexHeader {
    pub fn new () -> IndexHeader {
        IndexHeader {
            sign: [b'D', b'I', b'R', b'C'],
            version: 2,
            count: 0
        }
    }
    pub fn parse_header (data: &[u8]) -> IndexHeader {
        let sign: [u8; 4] = data[0..4].try_into().unwrap();
        assert_eq!(&sign, b"DIRC");
        let version: [u8; 4] = data[4..8].try_into().unwrap();
        let count: [u8; 4] = data[8..12].try_into().unwrap();
        let header = IndexHeader {
            sign: sign,
            version: u32::from_be_bytes(version),
            count: u32::from_be_bytes(count)
        };
        header
    }

    pub fn to_bytes (&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&self.sign);
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&self.count.to_be_bytes());
        bytes
    }
}

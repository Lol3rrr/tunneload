#[derive(Debug, PartialEq)]
pub struct Chunk {
    size: usize,
    body: Vec<u8>,
}

impl Chunk {
    pub fn new(size: usize, data: Vec<u8>) -> Self {
        Self { size, body: data }
    }

    pub fn serialize(&self, buf: &mut Vec<u8>) {
        let length = format!("{:x}", self.size);
        buf.extend_from_slice(length.as_bytes());
        buf.extend_from_slice("\r\n".as_bytes());
        buf.extend_from_slice(&self.body);
        buf.extend_from_slice("\r\n".as_bytes());
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[test]
fn serialize_valid() {
    let tmp = Chunk::new(9, "Developer".as_bytes().to_vec());

    let mut buf: Vec<u8> = Vec::new();
    tmp.serialize(&mut buf);

    assert_eq!("9\r\nDeveloper\r\n".as_bytes().to_vec(), buf);
}
#[test]
fn serialize_valid_2() {
    let tmp = Chunk::new(
        55,
        "This is just some random Data to fill the Response with"
            .as_bytes()
            .to_vec(),
    );

    let mut buf: Vec<u8> = Vec::new();
    tmp.serialize(&mut buf);

    assert_eq!(
        "37\r\nThis is just some random Data to fill the Response with\r\n"
            .as_bytes()
            .to_vec(),
        buf
    );
}

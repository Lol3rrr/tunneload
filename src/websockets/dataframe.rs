use crate::acceptors::traits::Receiver;

/// The Websocket Dataframe
#[derive(Debug, PartialEq)]
pub struct DataFrame {
    fin: bool,
    rsv_1: bool,
    rsv_2: bool,
    rsv_3: bool,
    opcode: u8,
    mask: bool,
    payload_len: u64,
    masking_key: Option<u32>,
    payload: Vec<u8>,
}

impl DataFrame {
    /// Attempts to receive an entire Dataframe from the given Receiver
    pub async fn receive<R>(rx: &mut R) -> Option<Self>
    where
        R: Receiver + Send,
    {
        let mut byte_buffer: [u8; 1] = [0];

        let (fin, rsv_1, rsv_2, rsv_3, opcode) = match rx.read(&mut byte_buffer).await {
            Ok(n) if n == 0 => return None,
            Err(e) => {
                log::error!("Receiving DataFrame: {:?}", e);
                return None;
            }
            _ => {
                let first_byte = byte_buffer[0];

                let fin = (first_byte & 0b10000000) > 0;
                let rsv_1 = (first_byte & 0b01000000) > 0;
                let rsv_2 = (first_byte & 0b00100000) > 0;
                let rsv_3 = (first_byte & 0b00010000) > 0;
                let opcode = first_byte & 0x0f;

                (fin, rsv_1, rsv_2, rsv_3, opcode)
            }
        };

        let (mask, initial_size) = match rx.read(&mut byte_buffer).await {
            Ok(n) if n == 0 => return None,
            Err(e) => {
                log::error!("Receiving DataFrame: {:?}", e);
                return None;
            }
            _ => {
                let first_byte = byte_buffer[0];

                let mask = (first_byte & 0b10000000) > 0;
                let initial_size = first_byte & 0b01111111;

                (mask, initial_size)
            }
        };

        let payload_len = match initial_size {
            126 => {
                let mut two_byte_buffer: [u8; 2] = [0, 0];
                match rx.read_full(&mut two_byte_buffer).await {
                    Err(e) => {
                        log::error!("Receiving DataFrame: {:?}", e);
                        return None;
                    }
                    _ => {}
                };

                u16::from_be_bytes(two_byte_buffer) as u64
            }
            127 => {
                let mut eight_byte_buffer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                match rx.read_full(&mut eight_byte_buffer).await {
                    Err(e) => {
                        log::error!("Receiving DataFrame: {:?}", e);
                        return None;
                    }
                    _ => {}
                };

                u64::from_be_bytes(eight_byte_buffer)
            }
            _ => initial_size as u64,
        };

        let masking_key = if mask {
            let mut four_byte_buffer: [u8; 4] = [0, 0, 0, 0];
            match rx.read_full(&mut four_byte_buffer).await {
                Err(e) => {
                    log::error!("Receiving DataFrame: {:?}", e);
                    return None;
                }
                _ => {}
            };

            Some(u32::from_be_bytes(four_byte_buffer))
        } else {
            None
        };

        let mut payload = vec![0; payload_len as usize];
        match rx.read_full(&mut payload).await {
            Err(e) => {
                log::error!("Receiving DataFrame: {:?}", e);
                return None;
            }
            _ => {}
        };

        Some(DataFrame {
            fin,
            rsv_1,
            rsv_2,
            rsv_3,
            opcode,
            mask,
            payload_len,
            masking_key,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::acceptors::mocks::Receiver as MockReceiver;

    #[tokio::test]
    async fn valid_dataframe_7bit_payload() {
        let header = vec![0b00000001, 0b00001010];
        let payload = vec![0; 10];
        let mut mock_rx = MockReceiver::new();
        mock_rx.add_chunk(header);
        mock_rx.add_chunk(payload);

        let expected = Some(DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: false,
            payload_len: 10,
            masking_key: None,
            payload: vec![0; 10],
        });
        assert_eq!(expected, DataFrame::receive(&mut mock_rx).await);
    }

    #[tokio::test]
    async fn valid_dataframe_16bit_payload() {
        let header = vec![0b00000001, 0b01111110, 0x00, 0xff];
        let payload = vec![0; 0xff];
        let mut mock_rx = MockReceiver::new();
        mock_rx.add_chunk(header);
        mock_rx.add_chunk(payload);

        let expected = Some(DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: false,
            payload_len: 0xff,
            masking_key: None,
            payload: vec![0; 0xff],
        });
        assert_eq!(expected, DataFrame::receive(&mut mock_rx).await);
    }

    #[tokio::test]
    async fn valid_dataframe_64bit_payload() {
        let header = vec![
            0b00000001, 0b01111111, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00,
        ];
        let payload = vec![0; 0x0f0000];
        let mut mock_rx = MockReceiver::new();
        mock_rx.add_chunk(header);
        mock_rx.add_chunk(payload);

        let expected = Some(DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: false,
            payload_len: 0x0f0000,
            masking_key: None,
            payload: vec![0; 0x0f0000],
        });
        assert_eq!(expected, DataFrame::receive(&mut mock_rx).await);
    }

    #[tokio::test]
    async fn valid_dataframe_mask() {
        let header = vec![0b00000001, 0b10001010, 0x12, 0x34, 0x56, 0x78];
        let payload = vec![0; 10];
        let mut mock_rx = MockReceiver::new();
        mock_rx.add_chunk(header);
        mock_rx.add_chunk(payload);

        let expected = Some(DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: true,
            payload_len: 10,
            masking_key: Some(0x12345678),
            payload: vec![0; 10],
        });
        assert_eq!(expected, DataFrame::receive(&mut mock_rx).await);
    }
}

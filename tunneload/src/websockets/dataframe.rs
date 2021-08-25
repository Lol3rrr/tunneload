use general_traits::Receiver;

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
                tracing::error!("Receiving DataFrame: {:?}", e);
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
                tracing::error!("Receiving DataFrame: {:?}", e);
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
                if let Err(e) = rx.read_full(&mut two_byte_buffer).await {
                    tracing::error!("Receiving DataFrame: {:?}", e);
                    return None;
                }

                u16::from_be_bytes(two_byte_buffer) as u64
            }
            127 => {
                let mut eight_byte_buffer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                if let Err(e) = rx.read_full(&mut eight_byte_buffer).await {
                    tracing::error!("Receiving DataFrame: {:?}", e);
                    return None;
                }

                u64::from_be_bytes(eight_byte_buffer)
            }
            _ => initial_size as u64,
        };

        let masking_key = if mask {
            let mut four_byte_buffer: [u8; 4] = [0, 0, 0, 0];
            if let Err(e) = rx.read_full(&mut four_byte_buffer).await {
                tracing::error!("Receiving DataFrame: {:?}", e);
                return None;
            }

            Some(u32::from_be_bytes(four_byte_buffer))
        } else {
            None
        };

        let mut payload = vec![0; payload_len as usize];
        if let Err(e) = rx.read_full(&mut payload).await {
            tracing::error!("Receiving DataFrame: {:?}", e);
            return None;
        }

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

    /// Serializes the DataFrame again in a the correct Format
    /// to be send out over the network again
    pub fn serialize(&self) -> Vec<u8> {
        let min_size = 0;
        let mut result = Vec::with_capacity(min_size);

        let mut first_byte = self.opcode & 0b00001111;
        if self.fin {
            first_byte |= 0b10000000;
        }
        if self.rsv_1 {
            first_byte |= 0b01000000;
        }
        if self.rsv_2 {
            first_byte |= 0b00100000;
        }
        if self.rsv_3 {
            first_byte |= 0b00010000;
        }
        result.push(first_byte);

        let mut second_byte: u8 = 0b00000000;
        if self.mask {
            second_byte |= 0b10000000;
        }
        if self.payload_len < 126 {
            second_byte |= (self.payload_len as u8) & 0b01111111;
            result.push(second_byte);
        } else if self.payload_len < u16::MAX as u64 {
            second_byte |= 0b01111110;
            result.push(second_byte);
            result.extend_from_slice(&(self.payload_len as u16).to_be_bytes());
        } else {
            second_byte |= 0b01111111;
            result.push(second_byte);
            result.extend_from_slice(&self.payload_len.to_be_bytes());
        }

        if let Some(masking_key) = self.masking_key {
            result.extend_from_slice(&masking_key.to_be_bytes());
        }

        result.extend_from_slice(&self.payload);

        result
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

    #[test]
    fn serialize_complete_7bit() {
        let tmp = DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: false,
            payload_len: 10,
            masking_key: None,
            payload: vec![0; 10],
        };

        let mut expected: Vec<u8> = vec![0; 12];
        expected[0] = 0b00000001;
        expected[1] = 0b00001010;

        assert_eq!(expected, tmp.serialize());
    }
    #[test]
    fn serialize_complete_16bit() {
        let tmp = DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: false,
            payload_len: 0xff,
            masking_key: None,
            payload: vec![0; 0xff],
        };

        let mut expected: Vec<u8> = vec![0; 0xff + 4];
        expected[0] = 0b00000001;
        expected[1] = 0b01111110;
        expected[2] = 0x00;
        expected[3] = 0xff;

        assert_eq!(expected, tmp.serialize());
    }
    #[test]
    fn serialize_complete_64bit() {
        let tmp = DataFrame {
            fin: false,
            rsv_1: false,
            rsv_2: false,
            rsv_3: false,
            opcode: 1,
            mask: false,
            payload_len: 0x0f0000,
            masking_key: None,
            payload: vec![0; 0x0f0000],
        };

        let mut expected: Vec<u8> = vec![0; 0x0f0000 + 10];
        expected[0] = 0b00000001;
        expected[1] = 0b01111111;
        expected[2] = 0x00;
        expected[3] = 0x00;
        expected[4] = 0x00;
        expected[5] = 0x00;
        expected[6] = 0x00;
        expected[7] = 0x0f;
        expected[8] = 0x00;
        expected[9] = 0x00;

        assert_eq!(expected, tmp.serialize());
    }
}

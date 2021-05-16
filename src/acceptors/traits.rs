use async_trait::async_trait;

/// This Trait specifies an interface that the
/// Rest of the Codebase can use to send the Data
/// back to the User, without needing to differentiate
/// between having a normal Webserver serve the user or
/// a connection from Tunneler
#[async_trait]
pub trait Sender {
    /// Sends the given Piece of data
    async fn send(&mut self, data: Vec<u8>, length: usize);
}

/// This Trait specifies an interface that the Rest
/// of the Codebase can use to read from an existing
/// connection without needing to know about how this
/// is actually done or through what acceptor this goes
#[async_trait]
pub trait Receiver {
    /// Reads from the Connection until there is either no more
    /// data left to read or until the provided Buffer is full
    ///
    /// Returns:
    /// The number of bytes that were read from the connection
    /// and written into the provided Buffer
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;

    /// Reads until the given Buffer is full or an error was encountered
    async fn read_full(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        let mut index = 0;
        let mut left_to_read = buf.len();

        while left_to_read > 0 {
            let read = self.read(&mut buf[index..]).await?;
            index += read;
            left_to_read -= read;
        }

        Ok(())
    }
}

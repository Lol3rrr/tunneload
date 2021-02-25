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
///
/// Behaviour:
/// Reads all available Data up to the length of the Buffer
/// from the underlying interface and writes it into the
/// Buffer.
///
/// Returns the Amount of Bytes written into the Buffer
/// n == 0 if the underlying interface received an EOF
#[async_trait]
pub trait Receiver {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

//! Uses a simple local Webserver to accept Requests

mod server;
pub use server::{Server, WebAcceptor};

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::Receiver;

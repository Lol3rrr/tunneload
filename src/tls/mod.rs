//! Contains some Generic functions and structs to
//! establish and use TLS for an HTTPS connection

mod receiver;
pub use receiver::Receiver;

mod sender;
pub use sender::Sender;

mod create_sender_receiver;
pub use create_sender_receiver::create_sender_receiver;

mod config_manager;
pub use config_manager::ConfigManager;

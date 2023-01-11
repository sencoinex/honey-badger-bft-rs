mod error;
pub use error::Error;
pub type Result<T> = core::result::Result<T, Error>;

pub mod binary_values;
pub mod coin_name;
pub mod epoch;
pub mod message;
pub mod node;
pub mod session;
pub mod state;
pub mod validator;

mod procedure;
pub use procedure::*;

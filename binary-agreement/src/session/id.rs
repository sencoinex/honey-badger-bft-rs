use core::fmt;

pub trait SessionId: Clone + fmt::Display + fmt::Debug + Send + Sync {}
impl<ID> SessionId for ID where ID: Clone + fmt::Display + fmt::Debug + Send + Sync {}

//! This module handles importing all types that should be necessary for any
//! end user to work with scrap. These work off the assumption that most users
//! will be interacting via methods defined on the Cmd type or one of it's
//! derivative types, such as CmdGroup.

pub use crate::flag::{Action, Flag, Value, ValueType};
pub use crate::Cmd;

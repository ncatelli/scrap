/// Defines behaviors for traits that can default to a if not specified value.
pub use crate::Defaultable;

/// Defines behaviors for types that can dispatch an evaluator to a function.
pub use crate::Dispatchable;

/// Defines behaviors for evaluating an input to a given type.
pub use crate::Evaluatable;

/// Defines a marker trait for denoting Cmd-like types.
pub use crate::IsCmd;

/// Defines a marker trait for denoting flag-like types from non-flag types.
pub use crate::IsFlag;

/// Defines behaviors for associating help strings with a given type.
pub use crate::{Helpable, ShortHelpable};

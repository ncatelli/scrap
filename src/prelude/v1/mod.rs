/// Provides a wrapper around spanned matching argument values.
pub use crate::Value;

/// Defines behaviors for traits that can default to a if not specified value.
pub use crate::Defaultable;

/// Defines behaviors for types that can dispatch an evaluator to a function.
pub use crate::Dispatchable;

/// Defines behaviors for types that can dispatch an evaluator to a function
/// with passed arguments.
pub use crate::DispatchableWithArgs;

/// Defines behaviors for types that can dispatch an evaluator to a function
/// with additional help documentation.
pub use crate::DispatchableWithHelpString;

/// Defines behaviors for types that can dispatch an evaluator to a function
/// with both a generated helpstring and all unparsed args.
pub use crate::DispatchableWithHelpStringAndArgs;

/// Defines behaviors for evaluating an input to a given type.
pub use crate::Evaluatable;

/// Defines a marker trait for denoting Cmd-like types.
pub use crate::IsCmd;

/// Defines a marker trait for denoting flag-like types from non-flag types.
pub use crate::IsFlag;

/// Defines behaviors for associating help strings with a given type.
pub use crate::{Helpable, ShortHelpable};

pub use crate::PositionalArgumentValue;

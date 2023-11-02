use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug, Clone)]
pub struct InconsistentTypeError {
    caller: String,
    expect: String,
    found: String,
}

impl InconsistentTypeError {
    pub fn new<T>(caller: T, expect: T, found: T) -> Self
    where
        T: ToString,
    {
        Self {
            caller: caller.to_string(),
            expect: expect.to_string(),
            found: found.to_string(),
        }
    }
}

impl Display for InconsistentTypeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(
            fmt,
            "inconsistent type error raised while calling `{}`. expect {}, found {}",
            self.caller, self.expect, self.found
        )
    }
}

impl Error for InconsistentTypeError {}

#[macro_export]
macro_rules! inconsistent_type_err {
    ($exp: expr, $fnd: expr) => {{
        InconsistentTypeError::new(function_name!(), $exp, $fnd).into()
    }};
}

pub use inconsistent_type_err;

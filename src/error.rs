use std;
use std::fmt;

pub type GenError = Box<std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;

#[derive(Debug, Clone)]
pub struct CastError {
    from: String,
    to: String,
}

impl CastError {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        let from = from.into();
        let to = to.into();
        Self { from: from, to: to }
    }
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "cannot cast {} to {}", &self.from, &self.to)
    }
}

impl std::error::Error for CastError {}

#[derive(Debug, Clone)]
pub struct ArityError {
    actual_args_count: usize,
    expected_args_count: usize,
    is_vararg: bool,
    fn_name: String,
}

impl ArityError {
    pub fn new(
        expected: usize,
        actual: usize,
        is_vararg: bool,
        fn_name: impl Into<String>,
    ) -> Self {
        Self {
            expected_args_count: expected,
            actual_args_count: actual,
            is_vararg: is_vararg,
            fn_name: fn_name.into(),
        }
    }
}

impl fmt::Display for ArityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "wrong number of arguments ({}) passed to {}",
            self.actual_args_count, self.fn_name
        )
    }
}

impl std::error::Error for ArityError {}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    message: String,
}

impl SyntaxError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxError {}

#[derive(Debug, Clone)]
pub struct UndefinedSymbol {
    symbol_name: String,
    is_fn: bool,
}

impl UndefinedSymbol {
    pub fn new(symbol_name: impl Into<String>, is_fn: bool) -> Self {
        Self {
            symbol_name: symbol_name.into(),
            is_fn: is_fn,
        }
    }
}

impl fmt::Display for UndefinedSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "undefined {} {}",
            if self.is_fn { "function" } else { "symbol" },
            self.symbol_name
        )
    }
}

impl std::error::Error for UndefinedSymbol {}

#[derive(Debug, Clone)]
pub struct GenericError {
    message: String,
}

impl GenericError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GenericError {}

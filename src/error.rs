use std;
use std::fmt;

pub type GenError = Box<std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;

#[derive(Debug, Clone)]
pub struct CastError {
    message: String,
    from: String,
    to: String
}

impl CastError {
    pub fn new(from: String, to: String) -> CastError {
        CastError {
            message: format!("cannot cast {} to {}", &from, &to),
            from: from,
            to: to
        }
    }
}


impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CastError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct ArityError {
    message: String,
    actual_args_count: usize,
    expected_args_count: usize,
    fn_name: String
}

impl ArityError {
    pub fn new(expected: usize, actual: usize, fn_name: String) -> ArityError {
        ArityError {
            message: format!("wrong number of arguments ({}) passed to {}",
                             actual, &fn_name),
            actual_args_count: actual,
            expected_args_count: expected,
            fn_name: fn_name,
        }
    }
}

impl fmt::Display for ArityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ArityError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    message: String
}

impl SyntaxError {
    pub fn new(message: String) -> SyntaxError {
        SyntaxError {
            message: message
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct UndefinedSymbol {
    message: String,
    symbol_name: String,
    is_fn: bool
}

impl UndefinedSymbol {
    pub fn new(symbol_name: String, is_fn: bool) -> UndefinedSymbol {
        UndefinedSymbol {
            message: format!("undefined {} {}",
                             if is_fn { "function" } else { "symbol" },
                             &symbol_name),
            symbol_name: symbol_name,
            is_fn: is_fn
        }
    }
}

impl fmt::Display for UndefinedSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UndefinedSymbol {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct GenericError {
    message: String
}

impl GenericError {
    pub fn new(message: String) -> GenericError {
        GenericError {
            message: message
        }
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GenericError {
    fn description(&self) -> &str {
        &self.message
    }
}

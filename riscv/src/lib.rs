pub mod executor;
pub mod lex;
pub mod parse;

// vec! like syntax for a hashmap
#[macro_export]
macro_rules! map {
    ($($key:expr => $val:expr),* $(,)?) => {
        ::std::collections::HashMap::from([$(($key, $val),)*])
    };
}

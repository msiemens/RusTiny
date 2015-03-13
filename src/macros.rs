#[macro_export]
macro_rules! fatal(
    ($msg:expr, $($arg:expr),*) => (
        $crate::driver::fatal(format!($msg, $($arg),*))
    );

    ($msg:expr) => (
        {
            use std::borrow::ToOwned;
            $crate::driver::fatal($msg.to_owned())
        }
    );
);

#[macro_export]
macro_rules! fatal_at(
    ($msg:expr, $($arg:expr),*; $loc:expr) => (
        $crate::driver::fatal_at(format!($msg, $($arg),*), $loc)
    );

    ($msg:expr; $loc:expr) => (
        {
            use std::borrow::ToOwned;
            $crate::driver::fatal_at($msg.to_owned(), $loc)
        }
    );
);
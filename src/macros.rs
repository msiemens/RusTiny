#[macro_export]
macro_rules! fatal(
    ($msg:expr, $($arg:expr),*) => (
        $crate::driver::session().err(format!($msg, $($arg),*))
    );

    ($msg:expr) => (
        {
            use std::borrow::ToOwned;
            $crate::driver::session().err($msg.to_owned())
        }
    );
);

#[macro_export]
macro_rules! fatal_at(
    ($msg:expr, $($arg:expr),*; $loc:expr) => (
        $crate::driver::session().span_err(format!($msg, $($arg),*), $loc)
    );

    ($msg:expr; $loc:expr) => (
        {
            use std::borrow::ToOwned;
            $crate::driver::session().span_err($msg.to_owned(), $loc)
        }
    );
);
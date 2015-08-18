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


#[macro_export]
macro_rules! with_reset(
    ($val:expr, $tmp:expr, $f:block) => (
        {
            let old_value = $val;
            $val = $tmp;

            let block_val = $f;

            $val = old_value;

            block_val
        }
    );
);


#[macro_export]
macro_rules! connect {
    ($items:expr, $fmt:expr, $connector:expr) => (
        $items.iter().map(|t| format!($fmt, t)).collect::<Vec<_>>().join($connector)
    )
}


#[macro_export]
macro_rules! try_insert {
    ($map:expr, $key:expr, $val:expr) => (
        {
            use std::collections::hash_map::Entry;

            match $map.entry($key) {
                Entry::Occupied(_) => Err(()),
                Entry::Vacant(entry) => {
                    entry.insert($val);
                    Ok(())
                }
            }
        }
    )
}
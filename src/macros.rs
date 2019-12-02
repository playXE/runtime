#[macro_export]
macro_rules! debug {
    ($($t: expr),*) => {
        #[cfg(feature="debug_print")]
        dbg!($($t),*);
    };
}

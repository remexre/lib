/// An explicit trivial cast.
#[macro_export]
macro_rules! coerce {
    ($e:expr => $t:ty) => {{
        let x: $t = $e;
        x
    }};
}

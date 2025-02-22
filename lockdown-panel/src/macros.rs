#[macro_export]
macro_rules! export {
    ($(
        $(#[$meta:meta])*
        $name:ident $(,)?
    )*) => {
        $(
            mod $name;

            $(#[$meta])*
            pub use $name::*;
        )*
    };
}

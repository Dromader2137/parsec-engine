//! Module responsible for defining components.

/// Marks a type as a component. Used for simple data types.
pub trait Component: Copy + Clone + Send + Sync + Sized + 'static {}
pub use parsec_engine_macros::Component;

macro_rules! impl_component_for_primitives {
    ( $( $t:ty ),* ) => {
        $(
            impl Component for $t {}
        )*
    }
}

impl_component_for_primitives!(
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    bool,
    char,
    &'static str
);

pub trait Component: Clone + Send + Sync + Sized + 'static {}
pub use oxide_engine_macros::Component;

macro_rules! impl_component_for_primitives {
    ( $( $t:ty ),* ) => {
        $(
            impl Component for $t {}
        )*
    }
}

impl_component_for_primitives!(
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
    bool, char, String, &'static str
);


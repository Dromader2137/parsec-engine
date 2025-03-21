use std::any::TypeId;
use super::{archetype::Archetype, WorldError};

pub trait Bundle: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
}

pub trait IntoArchetype: 'static {
    fn archetype() -> Result<Archetype, WorldError>;
}

macro_rules! impl_into_archetype {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<$t: Send + Sync + 'static> IntoArchetype for ($t, ) {
            fn archetype() -> Result<Archetype, WorldError> {
                Archetype::new(
                    vec![
                        std::any::TypeId::of::<$t>()
                    ]
                )
            }
        } 
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<$($t: Send + Sync + 'static),*> IntoArchetype for ($($t),*) {
            fn archetype() -> Result<Archetype, WorldError> {
                Archetype::new(
                    vec![
                        $(
                            std::any::TypeId::of::<$t>()
                        ),*
                    ]
                )
            }
        } 
    };
}

macro_rules! impl_bundle {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<$t: Sized + Send + Sync + 'static> Bundle for ($t, ) {
            fn type_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }
        }
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<$($t: Sized + Send + Sync + 'static),*> Bundle for ($($t),*) {
            fn type_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }
        }
    };
}

macro_rules! smaller_tuples_too {
    ($m: ident, $next: ident) => {
        $m!{$next}
        $m!{}
    };
    ($m: ident, $next: ident, $($rest: ident),*) => {
        $m!{$next, $($rest),*}
        smaller_tuples_too!{$m, $($rest),*}
    };
}


smaller_tuples_too!(impl_into_archetype, Z, Y, X, W, V, U, T, S, R, Q, P, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
smaller_tuples_too!(impl_bundle, Z, Y, X, W, V, U, T, S, R, Q, P, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);

#[cfg(test)]
mod test {
    use oxide_engine_macros::extract_tuple;

    #[test]
    fn test_extract_macro() {
        let a = (1.0_f32, 0.0_f64, "abc", 5_u8);
        let b: (f32, &str) = extract_tuple!(a, 0, 2);
        assert_eq!(b, (1.0_f32, "abc"))
    }
}

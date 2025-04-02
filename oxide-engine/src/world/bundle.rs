use std::any::TypeId;
use std::collections::HashSet;
use super::archetype::{ArchetypeId, Archetype};
use crate::oxide_engine_macros::{impl_bundle, impl_from_columns};

pub trait Bundle: Clone + Send + Sync + Sized + 'static {
    fn type_id(&self) -> TypeId;
    fn add_to(&self, arch: &mut Archetype);
}

pub trait FromColumns: Clone + Send + Sync + Sized + 'static {
    fn from_columns(arch: &Archetype) -> Vec<Self>;
}

pub trait IntoArchetypeId: Clone + Send + Sync + Sized + 'static {
    fn archetype_id() -> ArchetypeId;
}

macro_rules! hset {
    () => {
        HashSet::new()
    };
    ($($v:expr),*) => {
        {
            let mut hset = HashSet::new();
            $(
                hset.insert($v);
            )*
            hset
        }
    }
}

macro_rules! impl_into_archetype {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<$t: Clone + Send + Sync + Sized + 'static> IntoArchetypeId for ($t, ) {
            fn archetype_id() -> ArchetypeId {
                ArchetypeId::new(
                    hset![
                        std::any::TypeId::of::<$t>()
                    ]
                )
            }
        } 
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<$($t: Clone + Send + Sync + Sized + 'static),*> IntoArchetypeId for ($($t),*) {
            fn archetype_id() -> ArchetypeId {
                ArchetypeId::new(
                    hset![
                        $(
                            std::any::TypeId::of::<$t>()
                        ),*
                    ]
                )
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
// smaller_tuples_too!(impl_from_columns, Z, Y, X, W, V, U, T, S, R, Q, P, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);


impl_from_columns!(A, B);

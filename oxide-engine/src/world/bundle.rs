use crate::world::archetype::ColumnStateWrapper;
use oxide_engine_macros::multiple_tuples;

use super::archetype::{Archetype, ArchetypeError, ArchetypeId};
use crate::oxide_engine_macros::{impl_bundle, impl_from_columns, impl_from_columns_mut};
use std::any::TypeId;

pub trait Bundle: Clone + Send + Sync + Sized + 'static {
    fn type_id(&self) -> TypeId;
    fn add_to(&self, arch: &mut Archetype) -> Result<(), ArchetypeError>;
}

pub trait FromColumns<'a>: Clone + Send + Sync + Sized + 'static {
    type Output;
    fn iter_from_columns<'b>(
        arch: &'b Archetype,
    ) -> Result<impl Iterator<Item = Self::Output>, ArchetypeError>
    where
        'b: 'a;
}

pub trait FromColumnsMut<'a>: Clone + Send + Sync + Sized + 'static {
    type Output;
    fn iter_from_columns<'b>(
        arch: &'b Archetype,
    ) -> Result<impl Iterator<Item = Self::Output>, ArchetypeError>
    where
        'b: 'a;
}

pub trait IntoArchetypeId: Clone + Send + Sync + Sized + 'static {
    fn archetype_id() -> ArchetypeId;
}

pub trait UsableBundle<'a>: Bundle + FromColumns<'a> + IntoArchetypeId {}
pub trait UsableBundleMut<'a>: Bundle + FromColumnsMut<'a> + IntoArchetypeId {}

macro_rules! hset {
    () => {
        ::std::collections::HashSet::new()
    };
    ($($v:expr),*) => {
        {
            let mut hset = ::std::collections::HashSet::new();
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
                        ::std::any::TypeId::of::<$t>()
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
                            ::std::any::TypeId::of::<$t>()
                        ),*
                    ]
                )
            }
        }
    };
}

macro_rules! impl_usable_bundle {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<'a, $t: Clone + Send + Sync + Sized + 'static> UsableBundle<'a> for ($t, ) {}
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<'a, $($t: Clone + Send + Sync + Sized + 'static),*> UsableBundle<'a> for ($($t),*) {}
    };
}

macro_rules! impl_usable_bundle_mut {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<'a, $t: Clone + Send + Sync + Sized + 'static> UsableBundleMut<'a> for ($t, ) {}
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<'a, $($t: Clone + Send + Sync + Sized + 'static),*> UsableBundleMut<'a> for ($($t),*) {}
    };
}

multiple_tuples!(impl_into_archetype, 4);
multiple_tuples!(impl_bundle, 4);
multiple_tuples!(impl_from_columns, 4);
multiple_tuples!(impl_from_columns_mut, 4);
multiple_tuples!(impl_usable_bundle, 4);
multiple_tuples!(impl_usable_bundle_mut, 4);

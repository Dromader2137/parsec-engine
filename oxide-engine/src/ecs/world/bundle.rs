use super::archetype::{Archetype, ArchetypeError, ArchetypeId, ColumnStateWrapper};
use crate::oxide_engine_macros::{impl_bundle, impl_from_columns, impl_from_columns_mut};
use oxide_engine_macros::multiple_tuples;
use std::{
    any::TypeId,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

pub trait Component: Clone + Send + Sync + Sized + 'static {}

impl<T: Clone + Send + Sync + Sized + 'static> Component for T {}

pub trait Bundle: Component {
    fn type_id(&self) -> TypeId;
    fn add_to(&self, arch: &mut Archetype) -> Result<(), ArchetypeError>;
}

pub struct RefGuard<'a, T> {
    pub value: &'a T,
    column_state: &'a ColumnStateWrapper,
}

impl<'a, T> RefGuard<'a, T> {
    pub fn new(value: &'a T, column_state: &'a ColumnStateWrapper) -> Result<RefGuard<'a, T>, ArchetypeError> {
        column_state.borrow()?;
        Ok(RefGuard { value, column_state })
    }
}

impl<'a, T> Deref for RefGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> Drop for RefGuard<'a, T> {
    fn drop(&mut self) {
        self.column_state.free().unwrap();
    }
}

impl<'a, T: Debug> Debug for RefGuard<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefGuard").field("value", self.value).finish()
    }
}

impl<'a, T: Display> Display for RefGuard<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct RefGuardMut<'a, T> {
    pub value: &'a mut T,
    column_state: &'a ColumnStateWrapper,
}

impl<'a, T> RefGuardMut<'a, T> {
    pub fn new(value: &'a mut T, column_state: &'a ColumnStateWrapper) -> Result<RefGuardMut<'a, T>, ArchetypeError> {
        column_state.borrow_mut()?;
        Ok(RefGuardMut { value, column_state })
    }
}

impl<'a, T> Deref for RefGuardMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for RefGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T> Drop for RefGuardMut<'a, T> {
    fn drop(&mut self) {
        self.column_state.free().unwrap();
    }
}

impl<'a, T: Debug> Debug for RefGuardMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefGuardMut").field("value", self.value).finish()
    }
}

impl<'a, T: Display> Display for RefGuardMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub trait FromColumns<'a>: Component {
    type Output;
    fn iter_from_columns<'b>(arch: &'b Archetype) -> Result<impl Iterator<Item = Self::Output>, ArchetypeError>
    where
        'b: 'a;
}

pub trait FromColumnsMut<'a>: Component {
    type Output;
    fn iter_from_columns<'b>(arch: &'b Archetype) -> Result<impl Iterator<Item = Self::Output>, ArchetypeError>
    where
        'b: 'a;
}

pub trait IntoArchetypeId: Clone + Send + Sync + Sized + 'static {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
}

pub trait UsableBundle<'a>: Bundle + FromColumns<'a> + IntoArchetypeId {}
pub trait UsableBundleMut<'a>: Bundle + FromColumnsMut<'a> + IntoArchetypeId {}

macro_rules! impl_into_archetype {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<$t: Component> IntoArchetypeId for ($t, ) {
            fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
                ArchetypeId::new(
                    vec![
                        TypeId::of::<$t>()
                    ]
                )
            }
        }
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<$($t: Component),*> IntoArchetypeId for ($($t),*) {
            fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
                ArchetypeId::new(
                    vec![
                        $(
                            TypeId::of::<$t>()
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
        impl<'a, $t: Component> UsableBundle<'a> for ($t, ) {}
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<'a, $($t: Component),*> UsableBundle<'a> for ($($t),*) {}
    };
}

macro_rules! impl_usable_bundle_mut {
    ($t:ident) => {
        #[allow(unused_parens)]
        impl<'a, $t: Component> UsableBundleMut<'a> for ($t, ) {}
    };
    ($($t:ident),*) => {
        #[allow(unused_parens)]
        impl<'a, $($t: Component),*> UsableBundleMut<'a> for ($($t),*) {}
    };
}

multiple_tuples!(impl_into_archetype, 16);
multiple_tuples!(impl_bundle, 16);
multiple_tuples!(impl_from_columns, 16);
multiple_tuples!(impl_from_columns_mut, 16);
multiple_tuples!(impl_usable_bundle, 16);
multiple_tuples!(impl_usable_bundle_mut, 16);

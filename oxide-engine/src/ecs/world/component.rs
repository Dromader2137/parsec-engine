pub trait Component: Clone + Send + Sync + Sized + 'static {}

impl<T: Clone + Send + Sync + Sized + 'static> Component for T {}

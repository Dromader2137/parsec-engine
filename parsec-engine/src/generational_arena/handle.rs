use std::marker::PhantomData;

pub struct Handle<T, I = u32> {
    id: I,
    generation: I,
    _marker: PhantomData<T>
}

impl<T, I: Copy> Handle<T, I> {
    pub fn id(&self) -> I {
        self.id
    }

    pub fn generation(&self) -> I {
        self.generation
    }
}

pub struct WeakHandle<T, I = u32>(Handle<T, I>);

impl<T, I: Copy> WeakHandle<T, I> {
    pub fn id(&self) -> I {
        self.0.id
    }

    pub fn generation(&self) -> I {
        self.0.generation
    }
}

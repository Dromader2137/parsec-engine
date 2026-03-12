use std::{cell::{Ref, RefCell}, rc::Rc};

pub struct VulkanHandle<T> {
    object: Rc<RefCell<T>>,
}

impl<T> VulkanHandle<T> {
    pub fn new(object: T) -> Self {
        Self {
            object: Rc::new(RefCell::new(object))
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.object.borrow()
    }
}

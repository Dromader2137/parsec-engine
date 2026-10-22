// use std::{fmt::Debug, mem::MaybeUninit};

// A struct that allows you to store up to 8 elements and implements [`Copy`].
// #[derive(Debug, Clone)]
// struct SmallVec<T: Copy + Debug + Send + Sync + 'static> {
//     elements: [MaybeUninit<T>; 8],
//     len: u8
// }
//
// impl<T: Copy + Debug + Send + Sync + 'static> SmallVec<T> {
//     pub fn new() -> Self {
//         SmallVec { elements: [MaybeUninit::uninit(); 8], len: 0 }
//     }
// }
//
// impl<T: Copy + Debug + Send + Sync + 'static> Drop for SmallVec<T> {
//     fn drop(&mut self) {
//         for i in 0..self.len {
//             drop(self.elements[i as usize])
//         }
//     }
// }

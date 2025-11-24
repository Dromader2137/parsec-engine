use crate::ecs::{
    system::SystemInput,
    world::{World, archetype::Archetype, fetch::Fetch},
};

pub struct Query<T: Fetch> {
    fetches: Vec<T::State>,
}

impl<T: Fetch> SystemInput for Query<T> {
    fn borrow<'world>(world: &'world World) -> Self {
        let archetype_id = T::archetype_id().unwrap();
        let archetypes = world.archetypes.iter().filter_map(|(id, arch)| {
            if id.contains(&archetype_id) {
                Some(arch)
            } else {
                None
            }
        });
        let fetches = archetypes.map(|arch| T::prepare(arch).unwrap()).collect();
        Query { fetches }
    }
}

impl<T: Fetch> Query<T> {
    pub fn into_iter<'a>(&'a self) -> QueryIter<'a, T> {
        let fetches = self.fetches.iter().map(|fetch| T::borrow(fetch.clone())).collect();
        QueryIter {
            outside_len: 0,
            inside_len: 0,
            outside_idx: 0,
            inside_idx: 0,
            released: false,
            fetches 
        }
    }
}

#[derive(Debug)]
pub struct QueryIter<'a, T: Fetch + 'static> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    released: bool,
    fetches: Vec<T::Arr<'a>>,
}

impl<'a, T: Fetch + 'static> Iterator for QueryIter<'a, T> {
    type Item = T::Item<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let arr = &self.fetches[self.outside_idx];
        Some(T::get(arr, self.inside_idx))
    }
}

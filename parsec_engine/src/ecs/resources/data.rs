use std::{
    any::{Any, TypeId},
    collections::HashSet,
    sync::Arc,
};

use crate::{ecs::resources::ResourceMarker, utils::borrowing::BorrowingStats};

#[derive(Debug)]
pub(super) struct ResourceData {
    pub data: Box<dyn Any + Send + Sync + 'static>,
    pub borrowing_stats: Arc<BorrowingStats>,
    pub dependencies: HashSet<TypeId>,
    pub depended_on: HashSet<TypeId>,
}

impl ResourceData {
    pub fn new_any(resource: Box<dyn ResourceMarker>) -> ResourceData {
        let any_resource = resource.as_any();
        ResourceData {
            data: any_resource,
            borrowing_stats: Arc::new(BorrowingStats::default()),
            dependencies: HashSet::new(),
            depended_on: HashSet::new(),
        }
    }
}

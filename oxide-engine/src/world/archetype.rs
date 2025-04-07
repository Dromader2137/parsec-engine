#![allow(dead_code)] // To silence warnings about unused code in the example

use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap, BTreeSet}, // Using BTreeSet for deterministic order
    fmt, // Needed for Debug implementation
};

// --- Column Trait and Implementation ---

/// Trait for a type-erased column of component data.
/// Requires `Any` for downcasting and `Send + Sync` for potential parallel processing.
pub trait Column: Any + Send + Sync {
    /// Returns the number of components in the column.
    fn len(&self) -> usize;

    /// Returns `true` if the column is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes the component at `index`, replacing it with the last component
    /// to maintain dense storage. Returns the removed component (type-erased).
    /// Panics if `index` is out of bounds.
    fn swap_remove_dyn(&mut self, index: usize) -> Box<dyn Any + Send + Sync>;

    /// Removes all components from the column.
    fn clear(&mut self);

    /// Provides immutable access to the underlying `Any` type for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Provides mutable access to the underlying `Any` type for downcasting.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

/// A concrete column storing components of a specific type `T`.
struct TypedColumn<T: Any + Send + Sync + 'static> {
    data: Vec<T>,
}

impl<T: Any + Send + Sync + 'static> Column for TypedColumn<T> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    fn swap_remove_dyn(&mut self, index: usize) -> Box<dyn Any + Send + Sync> {
        // `Vec::swap_remove` efficiently removes an element by replacing it
        // with the last element and returns the removed element.
        Box::new(self.data.swap_remove(index))
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T: Any + Send + Sync + 'static> TypedColumn<T> {
    /// Provides direct access to the underlying `Vec<T>`.
    /// Caution: Ensure consistency with Archetype state if used directly.
    #[inline]
    fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Provides direct mutable access to the underlying `Vec<T>`.
    /// Caution: Ensure consistency with Archetype state if used directly.
    #[inline]
    fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }
}


// --- ArchetypeId ---

/// Uniquely identifies an Archetype based on the sorted set of component types it contains.
/// Using `BTreeSet` guarantees a canonical, ordered representation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] // Added necessary derives
pub struct ArchetypeId {
    component_types: BTreeSet<TypeId>, // Switched to BTreeSet for canonical ordering
}

impl ArchetypeId {
    /// Creates a new ArchetypeId from a set of component TypeIds.
    /// The set will be sorted internally by `BTreeSet`.
    pub fn new(component_types: BTreeSet<TypeId>) -> ArchetypeId {
        ArchetypeId { component_types }
    }

    /// Checks if this ArchetypeId contains all component types present in `other_id`.
    /// Useful for queries (e.g., "does this archetype satisfy the query requirements?").
    pub fn is_superset_of(&self, other_id: &ArchetypeId) -> bool {
        self.component_types.is_superset(&other_id.component_types)
    }

    /// Returns an iterator over the component TypeIds in this ArchetypeId (sorted).
    pub fn components(&self) -> impl Iterator<Item = &TypeId> {
        self.component_types.iter()
    }

     /// Returns the number of distinct component types in this archetype.
     pub fn component_count(&self) -> usize {
        self.component_types.len()
    }
}

// --- Archetype ---

/// Stores the component data for all entities sharing the same set of component types (ArchetypeId).
/// Data is stored in type-erased columns (`Box<dyn Column>`).
pub struct Archetype {
    /// The unique identifier defining the set of components stored here.
    pub id: ArchetypeId,
    /// Type-erased columns keyed by component `TypeId`.
    columns: HashMap<TypeId, Box<dyn Column>>,
    /// The number of entities (component bundles) stored in this archetype.
    /// Kept private; modified by methods that ensure column consistency.
    entity_count: usize, // Renamed for clarity, use usize
}

impl Archetype {
    /// Creates a new, empty Archetype for the given `ArchetypeId`.
    /// Initializes empty columns for all component types defined in the `id`.
    ///
    /// # Panics
    /// Panics in debug mode if a column cannot be created for a `TypeId` in the `id`.
    pub fn new(id: ArchetypeId) -> Archetype {
        let mut columns = HashMap::with_capacity(id.component_count());
        // Note: This requires a way to construct a `Box<dyn Column>` from just a TypeId.
        // This is tricky without reflection or macros. A common approach is to pass
        // constructors or use a factory pattern.
        // For this example, we'll assume columns are added differently or require
        // explicit construction for each type initially.
        // Let's simplify: Assume new creates the structure, columns are added via specific methods.
        // OR pass constructors:
        // pub fn new<I>(id: ArchetypeId, component_constructors: I) -> Archetype where I: IntoIterator...

        // Simplified `new`: Columns will be created on first access/add if using `entry` API later.
        // Or require explicit initial population. Let's require explicit population for safety:

        Archetype {
            id,
            columns: HashMap::new(), // Start empty, require population
            entity_count: 0,
        }
    }

    /// Creates an empty column for a specific type `T`. Helper function.
    /// Consider making this part of the `World` or a `ColumnFactory`.
    pub fn create_empty_column<T: Any + Send + Sync + 'static>() -> Box<dyn Column> {
        Box::new(TypedColumn::<T> { data: Vec::new() })
    }

    /// Adds an empty column for the specified type `T` if it doesn't exist.
    /// Usually called during initialization or when an archetype structure changes.
    /// Panics if the type `T` is not part of this archetype's `ArchetypeId`.
    pub fn ensure_column<T: Any + Send + Sync + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        assert!(self.id.component_types.contains(&type_id),
            "Attempted to add column for type not belonging to this ArchetypeId");

        self.columns.entry(type_id).or_insert_with(|| {
            Self::create_empty_column::<T>()
        });
    }

    /// Returns the number of entities (rows) stored in this archetype.
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_count
    }

    /// Returns `true` if this archetype contains no entities.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_count == 0
    }

    /// Gets an immutable slice of all components of type `T`.
    /// Returns `None` if this archetype does not contain type `T`.
    ///
    /// # Panics
    /// Panics if the internal type mapping is corrupt (i.e., the `TypeId` key
    /// in the `columns` map doesn't match the actual type within the `Box<dyn Column>`).
    /// This indicates a critical internal bug.
    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<&[T]> {
        self.columns
            .get(&TypeId::of::<T>())
            .map(|column_dyn| {
                column_dyn
                    .as_any()
                    .downcast_ref::<TypedColumn<T>>()
                    .expect("Internal type mismatch: TypeId key does not match Column type")
                    .data()
                    .as_slice()
            })
    }

    /// Gets a mutable slice of all components of type `T`.
    /// Returns `None` if this archetype does not contain type `T`.
    ///
    /// # Panics
    /// Panics if the internal type mapping is corrupt (see `get`).
    pub fn get_mut<T: Any + Send + Sync + 'static>(&mut self) -> Option<&mut [T]> {
        self.columns
            .get_mut(&TypeId::of::<T>())
             .map(|column_dyn| {
                column_dyn
                    .as_mut_any()
                    .downcast_mut::<TypedColumn<T>>()
                    .expect("Internal type mismatch: TypeId key does not match Column type")
                    .data_mut()
                    .as_mut_slice()
             })
    }

    /// Gets mutable access to the raw `Vec<T>` for component type `T`.
    /// Returns `None` if the column for `T` doesn't exist.
    ///
    /// # Safety / Intended Use
    /// This is a lower-level method, primarily intended for use by a managing system
    /// (e.g., a `World`) to implement atomic bundle additions. The caller *must* ensure:
    /// 1. Corresponding pushes/pops are performed on *all* relevant columns for an entity operation.
    /// 2. The `entity_count` is updated correctly *after* all column modifications are complete.
    /// Prefer safer, higher-level abstractions when possible.
    ///
    /// # Panics
    /// Panics on internal type mismatch (see `get`).
    pub(crate) fn get_column_vec_mut<T: Any + Send + Sync + 'static>(&mut self) -> Option<&mut Vec<T>> {
         self.columns
            .get_mut(&TypeId::of::<T>())
             .map(|column_dyn| {
                column_dyn
                    .as_mut_any()
                    .downcast_mut::<TypedColumn<T>>()
                    .expect("Internal type mismatch: TypeId key does not match Column type")
                    .data_mut()
             })
    }

    // --- Entity Management Methods ---

    /// Allocates space for a new entity, incrementing the entity count.
    /// Does *not* add any component data - assumes the caller will push data
    /// to the relevant columns using `get_column_vec_mut`.
    /// Returns the row index allocated for the new entity.
    pub(crate) fn allocate_entity_row(&mut self) -> usize {
        let index = self.entity_count;
        self.entity_count += 1;
        // Optional: Resize columns if needed, though Vec::push handles this.
        // Can add debug asserts here later.
        index
    }


    /// Removes the entity at the given `row_index` using swap_remove.
    ///
    /// This moves the component data of the *last* entity into the `row_index` slot
    /// to maintain dense arrays. It decrements the entity count.
    ///
    /// # Returns
    /// A `HashMap` containing the type-erased component data (`Box<dyn Any>`) of the
    /// entity that was originally at `row_index`.
    ///
    /// # Important
    /// The caller is responsible for updating any external `Entity` mappings.
    /// Specifically, the entity that was previously at the *last* row index
    /// (`old_last_index = self.entity_count` before decrement) now resides at `row_index`.
    /// The entity that was at `row_index` is gone.
    ///
    /// # Panics
    /// Panics if `row_index` is out of bounds (`>= self.len()`).
    pub fn swap_remove(&mut self, row_index: usize) -> HashMap<TypeId, Box<dyn Any + Send + Sync>> {
        assert!(row_index < self.entity_count, "swap_remove index out of bounds: index {} >= len {}", row_index, self.entity_count);

        let last_index = self.entity_count - 1;
        let mut removed_components = HashMap::with_capacity(self.columns.len());

        for (&type_id, column) in self.columns.iter_mut() {
            // `swap_remove_dyn` handles the swap and returns the element originally at `row_index`.
             // It internally calls Vec::swap_remove.
            let removed_component = column.swap_remove_dyn(row_index);
            removed_components.insert(type_id, removed_component);
        }

        // Decrement count *after* all swaps are done.
        self.entity_count -= 1;

        // Example assertion (optional, expensive):
        // debug_assert!(self.columns.values().all(|c| c.len() == self.entity_count), "Column lengths inconsistent after swap_remove");

        removed_components
    }

     /// Clears all component data from all columns and resets the entity count to zero.
     pub fn clear(&mut self) {
        for column in self.columns.values_mut() {
            column.clear();
        }
        self.entity_count = 0;
    }

    /// **Conceptual Note on Adding Entities:**
    /// The original code had an `add<T>` method which is insufficient for typical ECS patterns.
    /// An ECS adds a *bundle* of components for a single entity atomically.
    /// This involves:
    /// 1. Calling `allocate_entity_row()` to get the index for the new entity and increment the count.
    /// 2. Using `get_column_vec_mut::<T>()` for each component `T` in the bundle.
    /// 3. Pushing the actual component value onto the end of each respective `Vec<T>`.
    /// This entire process needs to be managed externally (e.g., by a `World` struct)
    /// to ensure all required columns are updated consistently for the allocated row.
}

// --- Debug Implementation for Archetype ---
impl fmt::Debug for Archetype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Archetype")
            .field("id", &self.id)
            .field("entity_count", &self.entity_count)
            // Avoid printing full column data, just the types
            .field("column_types", &self.columns.keys().collect::<Vec<_>>())
            .finish()
    }
}


// =======================================
// Example Usage & Tests
// =======================================
#[cfg(test)]
mod tests {
    use super::*;

    // Define some example components
    #[derive(Debug, Clone, PartialEq)] struct Position { x: f32, y: f32 }
    #[derive(Debug, Clone, PartialEq)] struct Velocity { dx: f32, dy: f32 }
    #[derive(Debug, Clone, PartialEq)] struct Mass(f32);
    #[derive(Debug, Clone, PartialEq)] struct Tag; // Zero-sized type example

    // Helper to create ArchetypeId from types
    fn create_archetype_id(types: &[TypeId]) -> ArchetypeId {
        ArchetypeId::new(types.iter().cloned().collect())
    }

    // Helper to create a populated archetype for testing
    fn create_test_archetype() -> Archetype {
        let types = [TypeId::of::<Position>(), TypeId::of::<Velocity>()];
        let id = create_archetype_id(&types);
        let mut archetype = Archetype::new(id);
        // Manually ensure columns exist for the test
        archetype.ensure_column::<Position>();
        archetype.ensure_column::<Velocity>();
        archetype
    }

    // Simulates adding a bundle (how a World might do it)
    fn add_bundle(archetype: &mut Archetype, pos: Position, vel: Velocity) -> usize {
        let entity_index = archetype.allocate_entity_row(); // Get index and increment count

        // Push data to corresponding columns - unwrap is safe if ensure_column was called
        archetype.get_column_vec_mut::<Position>().unwrap().push(pos);
        archetype.get_column_vec_mut::<Velocity>().unwrap().push(vel);

        // Verify lengths match entity count (optional debug check)
        debug_assert_eq!(archetype.get::<Position>().map_or(0, |s| s.len()), archetype.len());
        debug_assert_eq!(archetype.get::<Velocity>().map_or(0, |s| s.len()), archetype.len());

        entity_index
    }


    #[test]
    fn archetype_id_creation_and_compare() {
        let id_pv = create_archetype_id(&[TypeId::of::<Position>(), TypeId::of::<Velocity>()]);
        let id_p = create_archetype_id(&[TypeId::of::<Position>()]);
        let id_pv_clone = create_archetype_id(&[TypeId::of::<Velocity>(), TypeId::of::<Position>()]); // Order doesn't matter for BTreeSet content
        let id_pt = create_archetype_id(&[TypeId::of::<Position>(), TypeId::of::<Tag>()]);

        assert_eq!(id_pv, id_pv_clone);
        assert_ne!(id_pv, id_p);
        assert!(id_pv.is_superset_of(&id_p));
        assert!(id_pv.is_superset_of(&id_pv_clone)); // Superset of self
        assert!(!id_p.is_superset_of(&id_pv));
        assert!(!id_pv.is_superset_of(&id_pt));
    }

    #[test]
    fn archetype_new_and_empty() {
        let archetype = create_test_archetype();
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
        assert!(archetype.columns.contains_key(&TypeId::of::<Position>()));
        assert!(archetype.columns.contains_key(&TypeId::of::<Velocity>()));
    }

    #[test]
    fn simulate_bundle_add_and_get() {
        let mut archetype = create_test_archetype();

        let idx0 = add_bundle(&mut archetype, Position { x: 1.0, y: 2.0 }, Velocity { dx: 0.1, dy: 0.2 });
        let idx1 = add_bundle(&mut archetype, Position { x: 3.0, y: 4.0 }, Velocity { dx: 0.3, dy: 0.4 });

        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(archetype.len(), 2);

        // Test immutable get
        let positions = archetype.get::<Position>().unwrap();
        let velocities = archetype.get::<Velocity>().unwrap();
        let masses = archetype.get::<Mass>(); // Not in archetype

        assert_eq!(positions, &[Position { x: 1.0, y: 2.0 }, Position { x: 3.0, y: 4.0 }]);
        assert_eq!(velocities, &[Velocity { dx: 0.1, dy: 0.2 }, Velocity { dx: 0.3, dy: 0.4 }]);
        assert!(masses.is_none());

        // Test mutable get
        let positions_mut = archetype.get_mut::<Position>().unwrap();
        positions_mut[0].x = 10.0;
        assert_eq!(archetype.get::<Position>().unwrap()[0], Position { x: 10.0, y: 2.0 });
    }


    #[test]
    fn archetype_swap_remove() {
        let mut archetype = create_test_archetype();
        let _idx0 = add_bundle(&mut archetype, Position { x: 1.0, y: 1.0 }, Velocity { dx: 0.1, dy: 0.1 }); // Entity 0
        let _idx1 = add_bundle(&mut archetype, Position { x: 2.0, y: 2.0 }, Velocity { dx: 0.2, dy: 0.2 }); // Entity 1
        let _idx2 = add_bundle(&mut archetype, Position { x: 3.0, y: 3.0 }, Velocity { dx: 0.3, dy: 0.3 }); // Entity 2
        assert_eq!(archetype.len(), 3);

        // Remove entity at index 1 (Entity 1)
        let removed_data = archetype.swap_remove(1);
        assert_eq!(archetype.len(), 2);

        // Check removed data is from original index 1
        let removed_pos = removed_data.get(&TypeId::of::<Position>()).unwrap().downcast_ref::<Position>().unwrap();
        let removed_vel = removed_data.get(&TypeId::of::<Velocity>()).unwrap().downcast_ref::<Velocity>().unwrap();
        assert_eq!(*removed_pos, Position { x: 2.0, y: 2.0 });
        assert_eq!(*removed_vel, Velocity { dx: 0.2, dy: 0.2 });

        // Check current state: Entity 0 should be unchanged at index 0.
        // Entity 2 (originally last) should now be at index 1.
        let positions = archetype.get::<Position>().unwrap();
        let velocities = archetype.get::<Velocity>().unwrap();
        assert_eq!(positions.len(), 2);
        assert_eq!(velocities.len(), 2);

        // Entity 0 at index 0
        assert_eq!(positions[0], Position { x: 1.0, y: 1.0 });
        assert_eq!(velocities[0], Velocity { dx: 0.1, dy: 0.1 });

        // Entity 2 (was last) now at index 1
        assert_eq!(positions[1], Position { x: 3.0, y: 3.0 });
        assert_eq!(velocities[1], Velocity { dx: 0.3, dy: 0.3 });

         // Remove entity at index 0 (Entity 0)
         let removed_data_0 = archetype.swap_remove(0);
         assert_eq!(archetype.len(), 1);
         let removed_pos_0 = removed_data_0.get(&TypeId::of::<Position>()).unwrap().downcast_ref::<Position>().unwrap();
         assert_eq!(*removed_pos_0, Position { x: 1.0, y: 1.0 });

         // Check remaining data (Entity 2 should be at index 0)
          let positions = archetype.get::<Position>().unwrap();
          assert_eq!(positions, &[Position { x: 3.0, y: 3.0 }]);
    }

     #[test]
    fn archetype_clear() {
        let mut archetype = create_test_archetype();
        add_bundle(&mut archetype, Position { x: 1.0, y: 1.0 }, Velocity { dx: 0.1, dy: 0.1 });
        add_bundle(&mut archetype, Position { x: 2.0, y: 2.0 }, Velocity { dx: 0.2, dy: 0.2 });
        assert_eq!(archetype.len(), 2);

        archetype.clear();
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
        assert!(archetype.get::<Position>().map_or(true, |s| s.is_empty())); // Column exists but is empty
        assert!(archetype.get::<Velocity>().map_or(true, |s| s.is_empty()));
    }

    #[test]
    #[should_panic]
    fn swap_remove_out_of_bounds() {
        let mut archetype = create_test_archetype();
         add_bundle(&mut archetype, Position { x: 1.0, y: 1.0 }, Velocity { dx: 0.1, dy: 0.1 });
        archetype.swap_remove(1); // Index 1 is out of bounds (len is 1)
    }

     #[test]
    #[should_panic]
    fn ensure_column_wrong_type() {
         let types = [TypeId::of::<Position>()];
        let id = create_archetype_id(&types);
        let mut archetype = Archetype::new(id);
        archetype.ensure_column::<Velocity>(); // Velocity is not in ArchetypeId
    }
}

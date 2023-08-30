use crate::IndexDbEventRepository;
use cqrs_es::persist::PersistedEventStore;
use cqrs_es::CqrsFramework;

/// A convenience type for a CqrsFramework backed by
/// [IndexDbStore](struct.IndexDbStore.html).
pub type IndexDbCqrs<A> = CqrsFramework<A, PersistedEventStore<IndexDbEventRepository, A>>;

use std::marker::PhantomData;

// use async_trait::async_trait;
// use cqrs_es::persist::{PersistenceError, ViewContext, ViewRepository};
// use cqrs_es::{Aggregate, View};

// use crate::error::IndexDbAggregateError;

// /// A postgres backed query repository for use in backing a `GenericQuery`.
pub struct IndexDbViewRepository<V, A> {
    insert_sql: String,
    update_sql: String,
    select_sql: String,
    // pool: Pool<IndexDb>,
    _phantom: PhantomData<(V, A)>,
}

// impl<V, A> IndexDbViewRepository<V, A>
// where
//     V: View<A>,
//     A: Aggregate,
// {
//     /// Creates a new `IndexDbViewRepository` that will store serialized views in a IndexDb table named
//     /// identically to the `view_name` value provided. This table should be created by the user
//     /// before using this query repository (see `/db/init.sql` sql initialization file).
//     ///
//     /// ```
//     /// # use cqrs_es::doc::MyAggregate;
//     /// # use cqrs_es::persist::doc::MyView;
//     /// use sqlx::{Pool, IndexDb};
//     /// use postgres_es::IndexDbViewRepository;
//     ///
//     /// fn configure_view_repo(pool: Pool<IndexDb>) -> IndexDbViewRepository<MyView,MyAggregate> {
//     ///     IndexDbViewRepository::new("my_view_table", pool)
//     /// }
//     /// ```
//     pub fn new(view_name: &str) -> Self {
//         todo!()
//     }
// }

// #[async_trait]
// impl<V, A> ViewRepository<V, A> for IndexDbViewRepository<V, A>
// where
//     V: View<A>,
//     A: Aggregate,
// {
//     async fn load(&self, view_id: &str) -> Result<Option<V>, PersistenceError> {
//         todo!()
//     }

//     async fn load_with_context(
//         &self,
//         view_id: &str,
//     ) -> Result<Option<(V, ViewContext)>, PersistenceError> {
//         todo!()
//     }

//     async fn update_view(&self, view: V, context: ViewContext) -> Result<(), PersistenceError> {
//         todo!()
//     }
// }

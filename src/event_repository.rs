use crate::{js_event::JsEvent, IndexDbAggregateError};
use async_trait::async_trait;
use cqrs_es::persist::{
    PersistedEventRepository, PersistenceError, ReplayStream, SerializedEvent, SerializedSnapshot,
};
use cqrs_es::Aggregate;
use futures::channel::oneshot::channel;
use gloo_utils::format::JsValueSerdeExt;
use idb::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
// use futures::SinkExt;
use serde_json::Value;

/// An event repository relying on a IndexDb database for persistence.
pub struct IndexDbEventRepository {
    db_name: String,
    store_name: String,
}

#[async_trait]
impl PersistedEventRepository for IndexDbEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        self.select_events::<A>(aggregate_id).await
    }

    async fn get_last_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
        last_sequence: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_snapshot<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        todo!()
    }

    async fn persist<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
        snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        match snapshot_update {
            None => {
                self.insert_events::<A>(events).await?;
            }
            Some((aggregate_id, aggregate, current_snapshot)) => {
                // if current_snapshot == 1 {
                //     self.insert::<A>(aggregate, aggregate_id, current_snapshot, events)
                //         .await?;
                // } else {
                //     self.update::<A>(aggregate, aggregate_id, current_snapshot, events)
                //         .await?;
                // }
                todo!()
            }
        };
        Ok(())
    }

    async fn stream_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<ReplayStream, PersistenceError> {
        todo!()
    }

    // TODO: aggregate id is unused here, `stream_events` function needs to be broken up
    async fn stream_all_events<A: Aggregate>(&self) -> Result<ReplayStream, PersistenceError> {
        todo!()
    }
}

impl IndexDbEventRepository {
    async fn select_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        let (sender, receiver) = channel::<Vec<SerializedEvent>>();

        let db_name = self.db_name.clone();
        let store_name = self.store_name.clone();
        let aggregate_id = aggregate_id.to_string();

        spawn_local(async move {
            let db = connect(&db_name).await;

            // Create a transaction in readwrite mode
            let transaction = db
                .transaction(&[&store_name], TransactionMode::ReadWrite)
                .unwrap();

            // Get the object store
            let store = transaction.object_store(&store_name).unwrap();

            let index = store.index("aggregate_id").unwrap();

            let values = index
                .get_all(Some(Query::Key(aggregate_id.into())), None)
                .await
                .unwrap();

            let events: Vec<SerializedEvent> = values
                .into_iter()
                .map(|val| serde_wasm_bindgen::from_value::<JsEvent>(val).unwrap())
                .map(|js_event| js_event.into())
                .collect();

            sender.send(events).unwrap();
        });

        match receiver.await {
            Ok(result) => Ok(result),
            Err(_) => Err(PersistenceError::OptimisticLockError),
        }
    }
}

impl IndexDbEventRepository {
    pub fn new(db_name: Option<String>, store_name: Option<String>) -> Self {
        Self {
            db_name: db_name.unwrap_or("cqrs".to_string()),
            store_name: store_name.unwrap_or("events".to_string()),
        }
    }

    pub async fn insert_events<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
    ) -> Result<(), IndexDbAggregateError> {
        let (sender, receiver) = channel::<Result<(), IndexDbAggregateError>>();

        let db_name = self.db_name.clone();
        let store_name = self.store_name.clone();
        let events = events.to_vec();

        spawn_local(async move {
            let db = connect(&db_name).await;
            let events: Vec<JsValue> = events
                .into_iter()
                .map(|e| JsEvent::from(e.clone()))
                .map(|e| JsValue::from_serde(&e).unwrap())
                .collect();

            // Create a transaction in readwrite mode
            let transaction = db
                .transaction(&[&store_name], TransactionMode::ReadWrite)
                .unwrap();

            // Get the object store
            let store = transaction.object_store(&store_name).unwrap();

            let mut res: Result<(), IndexDbAggregateError> = Ok(());
            // Add the values to the store
            for event in events {
                web_sys::console::log_1(&event);
                if store.add(&event, None).await.is_err() {
                    res = Err(IndexDbAggregateError::OptimisticLock);
                }
            }

            // Commit the transaction
            if res.is_ok() {
                transaction.commit().await.unwrap();
            }

            sender.send(res).unwrap();
        });

        match receiver.await {
            Ok(result) => result,
            Err(_) => Err(IndexDbAggregateError::OptimisticLock),
        }
    }
}

async fn connect(name: &str) -> Database {
    // Better error messages in debug mode
    console_error_panic_hook::set_once();

    // Get a factory instance from global scope
    let factory = Factory::new().unwrap();

    // Create an open request for the database
    let mut open_request = factory.open(name, Some(1)).unwrap();

    // Add an upgrade handler for database
    open_request.on_upgrade_needed(|event| {
        // Get database instance from event
        let database = event.database().unwrap();

        // Prepare object store params
        let mut store_params = ObjectStoreParams::new();
        store_params.key_path(Some(KeyPath::new_array(vec![
            "aggregate_type",
            "aggregate_id",
            "sequence",
        ])));

        // Create object store
        let store = database
            .create_object_store("events", store_params)
            .unwrap();

        // Create index on object store
        store
            .create_index("aggregate_id", KeyPath::new_single("aggregate_id"), None)
            .unwrap();
    });

    open_request.await.unwrap()
}

use async_trait::async_trait;
use cqrs_es::persist::{
    PersistedEventRepository, PersistenceError, ReplayStream, SerializedEvent, SerializedSnapshot,
};
use wasm_bindgen_futures::spawn_local;

use cqrs_es::Aggregate;
use futures::channel::oneshot;
use idb::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::IndexDbAggregateError;

/// An event repository relying on a IndexDb database for persistence.
pub struct IndexDbEventRepository {}

#[derive(Deserialize, Serialize)]
pub struct JsSerializedEvent {
    pub aggregate_id: String,
    pub sequence: usize,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_version: String,
    pub payload: Value,
    pub metadata: Value,
}

impl Into<SerializedEvent> for JsSerializedEvent {
    fn into(self) -> SerializedEvent {
        SerializedEvent {
            aggregate_id: self.aggregate_id,
            sequence: self.sequence,
            aggregate_type: self.aggregate_type,
            event_type: self.event_type,
            event_version: self.event_version,
            payload: self.payload,
            metadata: self.metadata,
        }
    }
}

impl From<SerializedEvent> for JsSerializedEvent {
    fn from(value: SerializedEvent) -> JsSerializedEvent {
        JsSerializedEvent {
            aggregate_id: value.aggregate_id,
            sequence: value.sequence,
            aggregate_type: value.aggregate_type,
            event_type: value.event_type,
            event_version: value.event_version,
            payload: value.payload,
            metadata: value.metadata,
        }
    }
}

// #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
// #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
// #[async_trait]
#[async_trait]
impl PersistedEventRepository for IndexDbEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        let (tx, rx) = oneshot::channel::<Vec<SerializedEvent>>();

        spawn_local(async move {
            let db = create_database().await.unwrap();

            let transaction = db
                .transaction(&["events"], TransactionMode::ReadOnly)
                .unwrap();

            let store = transaction.object_store("events").unwrap();

            let index_store = store.index("aggregate_id").unwrap().object_store();

            let events = index_store.get_all(None, None).await.unwrap();

            let events: Vec<SerializedEvent> = events
                .into_iter()
                .map(|event| serde_wasm_bindgen::from_value(event).unwrap())
                .map(|event: JsSerializedEvent| event.into())
                .collect();

            tx.send(events).unwrap();
        });

        // Wait for the result
        match rx.await {
            Ok(result) => Ok(result),
            Err(_) => Err(PersistenceError::OptimisticLockError),
        }
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
        todo!()
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
    pub fn new() -> Self {
        Self {}
    }

    pub async fn insert_events<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
    ) -> Result<(), IndexDbAggregateError> {
        let (tx, rx) = oneshot::channel::<()>();

        let js_events: Vec<JsSerializedEvent> =
            events.into_iter().map(|e| e.clone().into()).collect();

        spawn_local(async move {
            let db = create_database().await.unwrap();

            let transaction = db
                .transaction(&["events"], TransactionMode::ReadWrite)
                .unwrap();

            // Get the object store
            let store = transaction.object_store("employees").unwrap();

            for js_event in js_events {
                store
                    .add(&serde_wasm_bindgen::to_value(&js_event).unwrap(), None)
                    .await
                    .unwrap();
            }

            // Commit the transaction
            transaction.commit().await.unwrap();

            tx.send(()).unwrap();
        });

        // Wait for the result
        match rx.await {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexDbAggregateError::UnknownError(err.to_string())),
        }
    }
}

async fn create_database() -> Result<Database, Error> {
    let name = "cqrs";

    // Get a factory instance from global scope
    let factory = Factory::new()?;

    // Create an open request for the database
    let mut open_request = factory.open(name, Some(1)).unwrap();

    // Add an upgrade handler for database
    open_request.on_upgrade_needed(|event| {
        // Get database instance from event
        let database = event.database().unwrap();

        // Prepare object store params
        let mut store_params = ObjectStoreParams::new();
        store_params.auto_increment(true);
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

    // `await` open request
    open_request.await
}

// #[cfg(test)]
// mod test {
//     use cqrs_es::persist::PersistedEventRepository;

//     use crate::error::IndexDbAggregateError;
//     use crate::testing::tests::{
//         test_event_envelope, Created, SomethingElse, TestAggregate, TestEvent, Tested,
//     };
//     use crate::IndexDbEventRepository;

//     #[tokio::test]
//     async fn event_repositories() {
//         let id = uuid::Uuid::new_v4().to_string();
//         let event_repo: IndexDbEventRepository = IndexDbEventRepository::new();
//         let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
//         assert!(events.is_empty());

//         event_repo
//             .insert_events::<TestAggregate>(&[
//                 test_event_envelope(&id, 1, TestEvent::Created(Created { id: id.clone() })),
//                 test_event_envelope(
//                     &id,
//                     2,
//                     TestEvent::Tested(Tested {
//                         test_name: "a test was run".to_string(),
//                     }),
//                 ),
//             ])
//             .await
//             .unwrap();
//         let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
//         assert_eq!(2, events.len());
//         events.iter().for_each(|e| assert_eq!(&id, &e.aggregate_id));

//         // Optimistic lock error
//         let result = event_repo
//             .insert_events::<TestAggregate>(&[
//                 test_event_envelope(
//                     &id,
//                     3,
//                     TestEvent::SomethingElse(SomethingElse {
//                         description: "this should not persist".to_string(),
//                     }),
//                 ),
//                 test_event_envelope(
//                     &id,
//                     2,
//                     TestEvent::SomethingElse(SomethingElse {
//                         description: "bad sequence number".to_string(),
//                     }),
//                 ),
//             ])
//             .await
//             .unwrap_err();
//         match result {
//             IndexDbAggregateError::OptimisticLock => {}
//             _ => panic!("invalid error result found during insert: {}", result),
//         };

//         let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
//         assert_eq!(2, events.len());

//         // verify_replay_stream(&id, event_repo).await;
//     }
// }

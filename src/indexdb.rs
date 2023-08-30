// use wasm_bindgen_futures::spawn_local;
use anyhow::*;
use idb::*;
use wasm_bindgen::JsValue;

pub struct IndexDbDatabase {
    db: Database,
}

impl IndexDbDatabase {
    // Todo: Error handling
    pub async fn new(name: &str) -> Result<Self> {
        // Get a factory instance from global scope
        let factory = Factory::new().map_err(|_| anyhow!("Error creating factory"))?;

        // Create an open request for the database
        let mut open_request = factory
            .open(name, Some(1))
            .map_err(|_| anyhow!("Error opening database"))?;

        // Add an upgrade handler for database
        open_request.on_upgrade_needed(|event| {
            // Get database instance from event
            let database = event
                .database()
                .map_err(|_| anyhow!("Error getting database"))
                .unwrap();

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
                .map_err(|_| anyhow!("Error creating object store"))
                .unwrap();

            // Create index on object store
            store
                .create_index("aggregate_id", KeyPath::new_single("aggregate_id"), None)
                .map_err(|_| anyhow!("Error creating index"))
                .unwrap();
        });

        // `await` open request
        let db = open_request
            .await
            .map_err(|_| anyhow!("Error opening request"))?;

        Ok(IndexDbDatabase { db })
    }

    pub async fn push(&self, store_name: &str, values: &Vec<JsValue>) -> Result<()> {
        // Create a transaction in readwrite mode
        let transaction = self
            .db
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .map_err(|_| anyhow!("Error creating transaction"))?;

        // Get the object store
        let store = transaction
            .object_store(store_name)
            .map_err(|_| anyhow!("Error getting object store"))?;

        // Add the values to the store
        for value in values {
            store
                .add(value, None)
                .await
                .map_err(|_| anyhow!("Error adding value"))?;
        }

        // Commit the transaction
        transaction
            .commit()
            .await
            .map_err(|_| anyhow!("Error commiting the transaction"))?;

        Ok(())
    }
}

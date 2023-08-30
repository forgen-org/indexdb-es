use indexdb_es::IndexDbDatabase;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn create_database() {
    assert!(IndexDbDatabase::new("test").await.is_ok());
}

wasm_bindgen_test_configure!(run_in_browser);

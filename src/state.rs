use anyhow::anyhow;
use std::sync::LazyLock;

use openai_api_rs::v1::api::OpenAIClient;
use rusqlite::Connection;

use crate::env::ENV_VARS;

pub const BREAD_STATE: LazyLock<BreadState> = LazyLock::new(|| {
    let openai_client = match &ENV_VARS.llm_api_key {
        Some(api_key) => OpenAIClient::builder()
            .with_endpoint(ENV_VARS.llm_endpoint.as_str())
            .with_api_key(api_key)
            .build()
            .map_err(|e| anyhow!("failed building client {e:?}"))
            .unwrap(),
        None => OpenAIClient::builder()
            .with_endpoint(ENV_VARS.llm_endpoint.as_str())
            .build()
            .map_err(|e| anyhow!("failed building client {e:?}"))
            .unwrap(),
    };
    let db_connection = Connection::open("users.db").expect("Failed to open db");
    db_connection
        .execute(
            "CREATE TABLE IF NOT EXISTS breads (
            user_id INTEGER PRIMARY KEY,
            bread_name TEXT NOT NULL
        )",
            [],
        )
        .unwrap_or_else(|_| {
            eprintln!("Failed at table creation");
            0
        });
    BreadState {
        db_connection: Connection::open("users.db").expect("Failed to open db"),
        openai_client: openai_client,
    }
});

pub struct BreadState {
    pub db_connection: Connection,
    pub openai_client: OpenAIClient,
}

use anyhow::anyhow;
use std::sync::LazyLock;

use openai_api_rs::v1::api::OpenAIClient;
use rusqlite::Connection;

use crate::env::ENV_VARS;

#[allow(clippy::declare_interior_mutable_const)]
#[allow(clippy::borrow_interior_mutability_const)]
pub const BREAD_STATE: LazyLock<BreadState> = LazyLock::new(|| {
    let env_vars = &ENV_VARS;
    let openai_client = ENV_VARS.llm_api_key.clone().map_or_else(
        || {
            OpenAIClient::builder()
                .with_endpoint(env_vars.llm_endpoint.as_str())
                .build()
                .map_err(|e| anyhow!("failed building client {e:?}"))
                .unwrap()
        },
        |api_key| {
            OpenAIClient::builder()
                .with_endpoint(env_vars.llm_endpoint.as_str())
                .with_api_key(api_key)
                .build()
                .map_err(|e| anyhow!("failed building client {e:?}"))
                .unwrap()
        },
    );
    let db_connection = Connection::open(&env_vars.db_location).expect("Failed to open db");
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
        openai_client,
    }
});

pub struct BreadState {
    pub db_connection: Connection,
    pub openai_client: OpenAIClient,
}

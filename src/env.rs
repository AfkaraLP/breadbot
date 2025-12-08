use std::sync::LazyLock;

pub struct EnvVars {
    pub guild_id: u64,
    pub discord_token: String,
    pub llm_endpoint: String,
    pub model_name: String,
    pub llm_api_key: Option<String>,
}

pub const ENV_VARS: LazyLock<EnvVars> = LazyLock::new(|| EnvVars {
    guild_id: dotenv::var("GUILD_ID")
        .map(|v| v.parse::<u64>())
        .expect("No guild id found in .env")
        .expect("Guild id has to be valid u64"),
    discord_token: dotenv::var("DISCORD_TOKEN")
        .expect("Expected a discord token in the environment"),
    llm_endpoint: dotenv::var("OPENAI_ENDPOINT")
        .expect("Please specify a chat completion endpoint"),
    llm_api_key: dotenv::var("LLM_API_KEY").map(Option::Some).unwrap_or(None),
    model_name: dotenv::var("MODEL_NAME").expect("Please provide a model name"),
});

use std::collections::HashMap;
use std::fmt::Display;

#[allow(unused)]
use anyhow::anyhow;

use openai_api_rs::v1::chat_completion::chat_completion::ChatCompletionRequest;
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, MessageRole};
use openai_api_rs::v1::completion::CompletionRequest;
use rusqlite::{OptionalExtension, params};

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, EditInteractionResponse, EditMember,
};
use serenity::builder::CreateCommand;

use crate::env::ENV_VARS;
use crate::state::BREAD_STATE;

const SYSTEM_PROMPT: &str = "You are a professional pun writer that specialized in bread puns. you are very creative. Your response contains the name encapsulated in []. for example [name_1]. be sure to have a very creative name but have it still adjacent to the original name. and keep in mind the format as it is very important.  example(s):

user: Rewrite the name Bradix to be a bread related pun.
assistant: [Breadix].

user: Rewrite the name AlbyPro to be a bread related pun.
assistant: [AlbyDough].
";

/// AfkaraLP's User ID
const OWNER_ID: u64 = 387230392278712320;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> serenity::Result<()> {
    match interaction.user.id.get() {
        OWNER_ID => {
            rename_users(ctx, interaction).await?;
            interaction
                .create_followup(
                    ctx,
                    CreateInteractionResponseFollowup::new()
                        .ephemeral(true)
                        .content("Finished Renaming all users."),
                )
                .await?;
            eprintln!("Finished Renaming all users");
        }
        _ => {
            interaction
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("You are not allowed to use this command"),
                    ),
                )
                .await?;
        }
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("rename").description("Renames the people of the server")
}

async fn rename_users(ctx: &Context, interaction: &CommandInteraction) -> serenity::Result<()> {
    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Renaming users..."),
            ),
        )
        .await?;
    if let Some(guild_id) = interaction.guild_id {
        let users = guild_id
            .members(ctx, None, None)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|user| user.user.id.get() != OWNER_ID);
        let db = dump_database();

        for mut user in users {
            let current_name = user.user.name.as_str();
            let user_id = user.user.id.get();

            // ===
            // Uncomment this if you want the bot to be slow af
            // ===
            //
            // _ = interaction
            //     .edit_response(
            //         ctx,
            //         EditInteractionResponse::new().content(format!("Renaming {current_name}...")),
            //     )
            //     .await;

            let new_name: String = match db.get(&user_id) {
                Some(bread_name) => bread_name.to_string(),
                None => {
                    let generated_name = {
                        loop {
                            match generate_name(current_name).await {
                                Ok(generated_name) => break generated_name,
                                Err(e) => {
                                    eprintln!(
                                        "[Error] ({e}) when generating username. Retrying..."
                                    );
                                }
                            }
                        }
                    };
                    insert_name_to_database(user_id, &generated_name)
                        .map_err(|_| serenity::Error::Other("Failed inserting name into DB"))?;
                    generated_name
                }
            };
            eprintln!("Got new username: {new_name}!");
            let old_name = current_name.to_string();
            if user.nick.as_ref() == Some(&new_name) {
                eprintln!("Skipping {old_name} because name hasn't changed");
                continue;
            }

            if let Err(e) = user.edit(ctx, EditMember::new().nickname(new_name)).await {
                eprintln!("Failed to edit username of {} because: {e}", old_name);
            }
        }
    }
    Ok(())
}

fn dump_database() -> HashMap<u64, String> {
    let mut map = HashMap::new();

    let conn = &BREAD_STATE.db_connection;

    let mut stmt = match conn.prepare("SELECT user_id, bread_name FROM breads") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare statement: {}", e);
            return map;
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?))
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to query rows: {}", e);
            return map;
        }
    };

    for row_result in rows {
        if let Ok((user_id, bread_name)) = row_result {
            map.insert(user_id, bread_name);
        }
    }

    map
}
fn insert_name_to_database(user_id: u64, name: &str) -> rusqlite::Result<()> {
    BREAD_STATE.db_connection.execute(
        "INSERT OR REPLACE INTO breads (user_id, bread_name) VALUES (?1, ?2)",
        params![user_id, name],
    )?;
    Ok(())
}
fn get_name_from_database(user_id: u64) -> rusqlite::Result<Option<String>> {
    BREAD_STATE
        .db_connection
        .query_row(
            "SELECT bread_name FROM breads WHERE user_id = ?1",
            params![user_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
}

async fn generate_name(name: impl Display) -> serenity::Result<String> {
    let prompt = format!("Rewrite the name {name} to be a bread related pun.");
    let system_prompt = ChatCompletionMessage {
        role: MessageRole::system,
        content: Content::Text(SYSTEM_PROMPT.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };
    let user_prompt = ChatCompletionMessage {
        role: MessageRole::user,
        content: Content::Text(prompt),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };
    let request = ChatCompletionRequest::new(
        ENV_VARS.model_name.to_string(),
        vec![system_prompt, user_prompt],
    )
    .stop(vec!["]".into()]);

    #[allow(const_item_mutation)]
    let completion = BREAD_STATE
        .openai_client
        .chat_completion(request)
        .await
        .map_err(|_| serenity::Error::Other("Failed to get LLM Completion"))?;

    let message = completion
        .choices
        .get(0)
        .map(|v| v.message.clone().content.unwrap_or_default())
        .ok_or(serenity::Error::Other(
            "Failed to extract from LLM Completion",
        ))?;

    message
        .split("[")
        .nth(1)
        .map(ToOwned::to_owned)
        .ok_or(serenity::Error::Other("failed splitting llm message at ["))
}

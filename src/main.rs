mod commands;
mod env;
mod state;

use serenity::all::{EditMember, GuildMemberUpdateEvent, Member, Message};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

use crate::commands::remove_mrbreast_scan::delete_mr_beast_message;
use crate::commands::rename::dump_database;
use crate::env::ENV_VARS;

/// `AfkaraLP`'s User ID
pub const OWNER_ID: u64 = 387_230_392_278_712_320;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        if let Err(e) = delete_mr_beast_message(new_message, ctx.http).await {
            eprintln!("Failed deleting mr breast message because {e:#?}");
        }
    }

    async fn guild_member_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Member>,
        _new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        let event_member = event.user;
        let db = dump_database();
        let Some(correlating_bread_name) = db.get(&event_member.id.get()) else {
            return eprintln!("no name for user_id in db found");
        };
        let guild_id = event.guild_id;
        let Ok(mut fetched_member) = guild_id.member(&ctx, &event_member.id).await else {
            return eprintln!("didn't get guild member new member");
        };
        if fetched_member.nick.as_ref() == Some(correlating_bread_name) {
            return;
        }
        eprintln!("renaming {} since update was detected", event_member.name);
        _ = fetched_member
            .edit(ctx, EditMember::new().nickname(correlating_bread_name))
            .await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let env_vars = &ENV_VARS;
        let guild_id = GuildId::new(env_vars.guild_id);

        guild_id
            .set_commands(
                &ctx.http,
                vec![
                    commands::rename::register(),
                    commands::remove_mrbreast_scan::register(),
                    commands::trump::register(),
                ],
            )
            .await
            .expect("Failed registering a command");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "rename" => {
                    if let Err(e) = commands::rename::run(&ctx, &command).await {
                        eprintln!("Error running rename command: {e}");
                    }
                    None
                }
                "delete_mr_beast_messages" => {
                    if let Err(e) = commands::remove_mrbreast_scan::run(&ctx, &command).await {
                        eprintln!("Error running mrbeast command: {e}");
                    }
                    None
                }
                "trump" => {
                    if let Err(e) = commands::trump::run(&ctx, &command).await {
                        eprintln!("Error running trump command: {e}");
                    }
                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let env_vars = &ENV_VARS;
    let mut client = Client::builder(&env_vars.discord_token, GatewayIntents::privileged())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

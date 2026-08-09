use serenity::all::{
    CacheHttp, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, GetMessages, Message,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> serenity::Result<()> {
    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Deleting Mr. Least Scam Messages"),
            ),
        )
        .await?;

    let mut total_deleted_messages = 0;

    if let Some(guild) = interaction.guild_id
        && let Ok(channels) = guild.channels(ctx).await
    {
        for channel in channels.values() {
            if let Ok(messages) = channel.messages(ctx, GetMessages::new().limit(20)).await {
                for message in messages {
                    if delete_mr_beast_message(message, ctx).await?.is_some() {
                        total_deleted_messages += 1;
                    }
                }
            }
        }
    }
    interaction
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .ephemeral(true)
                .content(format!(
                    "Successfully deleted {total_deleted_messages} Mr. Least messages. (or other 4 attachment messages, my bad)"
                )),
        )
        .await?;
    Ok(())
}
pub fn register() -> CreateCommand {
    CreateCommand::new("delete_mr_beast_messages").description("take a wild friggin guess")
}

pub async fn delete_mr_beast_message(
    message: Message,
    ctx: impl CacheHttp,
) -> serenity::Result<Option<()>> {
    if message.attachments.len() != 4 {
        return Ok(None);
    }
    message.delete(ctx).await?;
    Ok(Some(()))
}

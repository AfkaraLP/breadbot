use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAttachment, CreateCommand,
    CreateCommandOption, CreateInteractionResponseMessage,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> serenity::Result<()> {
    let Some(phrase) = interaction
        .data
        .options
        .iter()
        .find(|i| i.name == "input")
        .map(|v| v.value.clone())
        .and_then(|v| v.as_str().map(ToOwned::to_owned))
    else {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "Please give the president actually something to say, or don't waste his time — sad.",
        );
        return Ok(());
    };

    if phrase.chars().count() > 256 {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "The president is a busy man, he only has time to say messages of 256 character length or less, or else he does not have enough time to make america great again, which is frankly quite sad.",
        );
        return Ok(());
    }

    let client = reqwest::Client::new();

    let Ok(response) = client
        .post("https://trump.afkara.dev/api/synthesize")
        .bearer_auth("trump67")
        .json(&std::collections::BTreeMap::from([("text", phrase)]))
        .send()
        .await
    else {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "Failed sending the request to the TTS server — Trump is in a busy meeting atm most likely.",
        );
        return Ok(());
    };
    let Ok(body) = response.json::<serde_json::Value>().await else {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "Could not get audio from server — Trump is probably sleeping right now.",
        );
        return Ok(());
    };
    let Some(audio_url) = body.get("audio_url").and_then(|value| value.as_str()) else {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "Could not get audio from server — Trump is probably sleeping right now.",
        );
        return Ok(());
    };
    let Ok(response) = client
        .get(format!("https://trump.afkara.dev{audio_url}"))
        .send()
        .await
    else {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "Could not get audio from server — Trump is probably sleeping right now.",
        );
        return Ok(());
    };
    let Ok(audio) = response.bytes().await else {
        _ = silently_say_trump_err(
            ctx,
            interaction,
            "Could not get audio from server — Trump is probably sleeping right now.",
        );
        return Ok(());
    };

    _ = interaction
        .create_response(
            ctx,
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Here you go — I said it — tremendous!")
                    .add_file(CreateAttachment::bytes(audio, "tremendous_audio.wav")),
            ),
        )
        .await;
    Ok(())
}
pub fn register() -> CreateCommand {
    CreateCommand::new("trump").add_option(CreateCommandOption::new(
        CommandOptionType::String,
        "input",
        "The phrase you want our great president to say — tremendous.",
    ))
    .description("Generate a Trump-style audio clip from text.")
}

async fn silently_say_trump_err<T: ToString>(
    ctx: &Context,
    interaction: &CommandInteraction,
    msg: T,
) -> serenity::Result<()> {
    interaction
        .create_response(
            ctx,
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(msg.to_string())
                    .ephemeral(true),
            ),
        )
        .await
}

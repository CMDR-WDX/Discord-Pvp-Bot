mod commands;
mod data;
use commands::{pvpself::pvpself, pvpwhois::pvpwhois, pvpregister::pvpregister};
use poise::serenity_prelude as serenity;

pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;



#[tokio::main]
async fn main() {
    // This will panic and fail is there is an Error Return value.
    data::startup_check().unwrap();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pvpself(), pvpwhois(), pvpregister()],
            ..Default::default()
        })
        .token(data::Environment::discord_token())
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id = serenity::GuildId(data::Environment::guild_id());
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
                Ok(Data {})
            })
        });
    framework.run().await.unwrap();
}
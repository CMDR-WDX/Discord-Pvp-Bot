mod commands;
use commands::{pvpself::pvpself, pvpwhois::pvpwhois};
use poise::serenity_prelude as serenity;

pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;



#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load data via dotenv.");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pvpself(), pvpwhois()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id_val: u64 = std::env::var("GGI_GUILD_ID").expect("missing GGI_GUILD_ID Env Var").parse().expect("Failed to parse content as integer");
                let guild_id = serenity::GuildId(guild_id_val);
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
                Ok(Data {})
            })
        });
    framework.run().await.unwrap();
}
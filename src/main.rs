mod commands;
mod data;
mod util;
mod other;
use commands::{pvpself::pvpself, pvpwhois::pvpwhois, pvpregister::pvpregister, pvpweekly::pvpweekly};
use poise::serenity_prelude as serenity;

use crate::{other::weekly_summary_event::subscribe_for_event, commands::admin::pvpadminforceweeklysummary::pvpadmin_force_weekly_summary};
use tokio_cron_scheduler::{JobScheduler, Job};


pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;



#[tokio::main]
async fn main() {
    // This will panic and fail is there is an Error Return value.
    data::startup_check().unwrap();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pvpself(), pvpwhois(), pvpregister(), pvpweekly(), pvpadmin_force_weekly_summary()],
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
    
    // Scheduling
    println!("Setting up Cron Scheduler...");
    let sched = JobScheduler::new().await.unwrap();

    let job = Job::new_async("0 0 8 * * Thu", | _uuid, mut _l| {
        Box::pin(async move {
            let _ = subscribe_for_event(None).await;
        })
    }).map_err(|x| x.to_string());   
    
    match job {
        Err(err) => println!("ERR: {}", err),
        Ok(val) => match sched.add(
            val
        ).await {
            Ok(_) => println!("Scheduler setup."),
            Err(err) => println!("{}", err.to_string()),
        }
    }

    let _ = sched.start().await;
    
    println!("Starting up Bot...");
    framework.run().await.unwrap();
}
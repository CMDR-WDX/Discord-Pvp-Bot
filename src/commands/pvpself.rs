use std::time::Duration;

use super::super::{Context, Error};


#[poise::command(slash_command)]
pub async fn pvpself(
    ctx: Context<'_>
) -> Result<(), Error> {
    let user = ctx.author();
    let handle = ctx.say(format!("You are {}. Your ID is {}",  user.tag(), user.id)).await?;
    tokio::time::sleep(Duration::new(2, 0)).await;
    handle.edit(ctx, |x| {
        x.content("Nevermind :)")
    }).await?;
    Ok(())
}
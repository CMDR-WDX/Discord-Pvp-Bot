use crate::{util::new_username::convert_tag_to_username, commands::admin::util::is_user_admin};

use super::super::{Context, Error};


/// Check if you have an Account with Pvp Bot.
#[poise::command(slash_command)]
pub async fn pvpself(
    ctx: Context<'_>
) -> Result<(), Error> {
    let user = ctx.author();
    let is_admin = is_user_admin(&ctx).await;
    let admin_str = match is_admin {
        true => "",
        false => "not "
    };
    let content = format!("You are {}.\nYour ID is {}\n You are {}an admin.",  convert_tag_to_username(user.tag())  , user.id, admin_str);
    match ctx.send(|x| x.ephemeral(true).content(content)).await {
        Ok(_) => {},
        Err(err) => println!("Error in pvpself command: {}", err),
    }
    return Ok(());


}
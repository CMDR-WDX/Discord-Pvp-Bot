

#[derive(Serialize, Deserialize, Debug)]
struct OkResponse {

    error: Option<String>,
    #[serde(rename="isBanned")]
    is_banned: Option<bool>,
    token: Option<String>
}



use std::time::Duration;

use poise::serenity_prelude::{Color, GuildId, RoleId, User};
use serde::{Serialize, Deserialize};

use crate::data::Environment;

use super::super::{Context, Error};

enum ErrorType {
    IsBanned,
    Other(String)
}


async fn do_request(user: &User) -> Result<String, ErrorType> {
    let request_form = reqwest::multipart::Form::new()
    .text("discord_name", user.tag())
    .text("discord_id", user.id.to_string());


    let response = reqwest::Client::new()
        .post(format!("{}/api/token/discord", Environment::server_address()))
        .multipart(request_form)
        .bearer_auth(Environment::server_auth())
        .send().await.map_err(|x| ErrorType::Other(x.to_string()))?;

    match response.status().is_success() {
        false => Err(ErrorType::Other(format!("Server responded with a {}", response.status().as_str()))),
        true => {
            // Parse as JSON
            let as_json: OkResponse = response.json().await.map_err(|x|ErrorType::Other(x.to_string()))?;
            
            match as_json.is_banned {
                Some(is_banned) => {
                    if is_banned {
                        return Err(ErrorType::IsBanned);
                    }
                },
                None => {}
            }

            match as_json.token {
                Some(token) => return Ok(token),
                None => return Err(ErrorType::Other("No Token was returned".to_string()))
            }
        }
    }
} 








/// Request an API Token. You can also use this command to get
#[poise::command(slash_command)]
pub async fn pvpregister(
    ctx: Context<'_>
) -> Result<(), Error> {

    let user = ctx.author();

    // First Check if the user has the relevant role
    let required_role_id = RoleId(Environment::role_auth());
    let required_guild_id = GuildId(Environment::guild_id());

    if !user.has_role(ctx.http(), required_guild_id, required_role_id).await? {
        // User does not have the needed Role
        ctx.send(|b| b.embed(|e| {
            e.color(Color::RED)
                .title(":octagonal_sign: No Permission")
                .description("You don't have permissions to get an API Key. Gank some plebs to get the perms needed.")
        })).await?;
        return Ok(());
    }

    // Try to create a new response
    let response = do_request(user).await;

    match response {
        Err(err) => {
            let heading = match err {
                ErrorType::IsBanned => ":octagonal_sign: User Banned",
                ErrorType::Other(_) => ":warning: Error occurred"
            };
            let description: String = match err {
                ErrorType::IsBanned => "Your Discord Account has been sanctioned. No API Key for you.".to_string(),
                ErrorType::Other(err) => format!("An Error occurred:\n{}", err)
            };
            ctx.send(|b|b.embed(|e| {
                e.color(Color::RED)
                    .title(heading)
                    .description(description)
            })).await?;
        },
        Ok(token) => {
            // Got a valid token from the Server
            let sleep_duration_seconds = 60;
            let message = ctx.send(|b|b.ephemeral(true).embed(|e| {
                e.color(Color::DARK_GREEN)
                    .title(":white_check_mark: New API Key created")
                    .description(format!("You have {}s to copy the key.\n\nOnly you can see this message.", sleep_duration_seconds))
                    .field("API Key", format!("`{}`", token), false)
            })).await?;
            tokio::time::sleep(Duration::from_secs(sleep_duration_seconds)).await;
            message.edit(ctx, |f| f.embed(|e| e.title(":white_check_mark: New API Key created").color(Color::BLUE).description("Key has been hidden. Rerun `/pvpregister` if you need a new key."))).await?;
        }
    }

    Ok(())
}
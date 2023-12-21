use chrono::NaiveDate;
use poise::serenity_prelude::Color;

use crate::other::weekly_summary_event::subscribe_for_event;
use crate::commands::admin::util::is_user_admin;

use crate::{Context, Error};

/// Check if you have an Account with Pvp Bot.
#[poise::command(slash_command)]
pub async fn pvpadmin_force_weekly_summary(
    ctx: Context<'_>,
    #[description="optional; Use YYYY-MM-DD format. If left empty, the current day is used."] for_date: Option<String>
) -> Result<(), Error> {
    let is_admin = is_user_admin(&ctx).await;

    let override_date: Option<NaiveDate> = match for_date {
        Some(e) => {
            match NaiveDate::parse_from_str(e.as_str(), "%Y-%m-%d").map_err(|x| x.to_string()) {
                Ok(date) => Some(date),
                Err(err) => {
                    ctx.send(|x| x.ephemeral(true).embed(|e| e.color(Color::RED).title("Error").description(err))).await?;
                    return Ok(());
                },
            }
        }
        None => None
    };


    let _ = match is_admin {
        false => {
            let request = ctx.send(|x| x.ephemeral(true).embed(
                |e| e.color(Color::RED).description("This command can only be invoked by admins").title("Ye shan't pass")
            )).await;

            (request, true)
        },
        true => {
            let response = subscribe_for_event(override_date).await;
            let embed_color =  match response {
                Ok(_) => Color::DARK_GREEN,
                Err(_) => Color::RED
            };
            let message = match response {
                Ok(_) => "Sent successfully".to_string(),
                Err(err) => {
                    err
                }
            };
            let request = ctx.send(|x| x.ephemeral(true).embed(|e| e.color(embed_color).description(message))).await;
            (request, false)
        }
    };



    return Ok(());



}
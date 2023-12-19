use poise::serenity_prelude::Color;

use crate::other::{weekly_summary::{get_kills_for_current_cycle, get_sorted_weekly_summary, get_relevant_time_range}, weekly_summary_event::{build_weekly_embeds, is_cmdr_in_result_set}};

use super::super::{Context, Error};

pub struct DataRow {
    pub position: u32,
    pub cmdr: String,
    pub kill_count: u32,
    pub highlight: bool
}


/// Gets the leaderboard for current game week (ends on Thursdays 8AM UTC)
#[poise::command(slash_command)]
pub async fn pvpweekly(
    ctx: Context<'_>,
    #[description="optional; Get Ranking for CMDR in the current leaderboard period"] cmdr_name: Option<String>
) -> Result<(), Error> {

    let response_from_backend = get_kills_for_current_cycle().await;
    let time_range_utc = get_relevant_time_range();
    let time_range_date = (time_range_utc.0.format("%Y-%m-%d"), time_range_utc.1.format("%Y-%m-%d"));

    let message = ctx.send(|builder| {
        builder.embed(|embed: &mut poise::serenity_prelude::CreateEmbed| {
            match response_from_backend {
                Err(err) => {
                    embed.color(Color::RED).title("Error").description(err)
                },
                Ok(data) => {
                    let result: Vec<(String, u32)> = get_sorted_weekly_summary(data);
                    
                    let data_to_present = match cmdr_name {
                        Some(ref name) => match is_cmdr_in_result_set(&result, &name) {
                            false => None,
                            true => {
                                let mut queue: Vec<(u32, &str, u32)> = vec![];
                                
                                for(i, (el_str, el_count)) in result.iter().enumerate() {
                                    queue.push((u32::try_from(i).unwrap(), el_str.as_str(), *el_count));
                                    if el_str.eq_ignore_ascii_case(&name) {
                                        break
                                    }
                                }

                                let data = queue.into_iter().rev().take(5).enumerate().map(|(i,e)| DataRow {
                                    position: e.0 + 1,
                                    cmdr: e.1.to_owned(),
                                    kill_count: e.2,
                                    highlight: i == 0
                                }).rev().collect::<Vec<_>>();
                                
                                Some(data)
                            }
                        },
                        None => {
                            let data = result.into_iter().enumerate().take(10).map(|(i,e)| DataRow {
                                position: u32::try_from(i).unwrap() + 1,
                                cmdr: e.0.to_owned(),
                                kill_count: e.1,
                                highlight: false
                            }).collect::<Vec<_>>();
                            Some(data)
                        },
                    };

                    fn create_not_found_error<'str_life>(name: &Option<String>, embed: &mut poise::serenity_prelude::CreateEmbed) {
                        let description = match name {
                            None => "Noone has made any kills for this leaderboard period… *yet.".to_string(),
                            Some(name) => {
                                let message = format!("CMDR {} has yet to get any kills in this leaderboard period.", name);
                                message
                            }
                        };

                        embed.color(Color::RED).title("Not Found").description(description);
                    }


                    match data_to_present {
                        None => create_not_found_error(&cmdr_name, embed),
                        Some(data) => {
                            build_weekly_embeds(data, embed);
                        },
                    };

                    embed.footer(|footer| footer.text(format!("Current leaderboard period is from {} to {}, starting at 8AM UTC.", time_range_date.0, time_range_date.1)));

                    embed
                },
            }
        })
    } ).await;
    
    if let Err(reason) = message {
        println!("Error Sending Message: {:?}", reason)
    }

    Ok(())
}

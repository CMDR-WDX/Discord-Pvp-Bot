use poise::serenity_prelude::{Http, ChannelId, Color};
use std::ops::{Add, Sub};
use chrono::{Datelike, Timelike, DateTime, Utc, Days};

use crate::{data::{self, Environment}, commands::pvpweekly::DataRow};

use super::weekly_summary::{fetch_from_server, get_query_string_for_api_call, get_sorted_weekly_summary};

pub async fn subscribe_for_event() -> Result<(), String> {
    let client = Http::new(&data::Environment::discord_token());

    let relevant_range = get_relevant_time_range_summary();
    let data = fetch_from_server(get_query_string_for_api_call(relevant_range.0, relevant_range.1)).await;


    let message_sent_result = ChannelId(Environment::discord_weekly_update_channel_id()).send_message(&client, |m| m.embed(|embed| {
        embed.title("The leaderboard period has ended.").footer(
            |footer| footer.text(format!("Current leaderboard period is from {} to {}, starting at 8AM UTC.", relevant_range.0.format("%Y-%m-%d"), relevant_range.1.format("%Y-%m-%d")))  
        );
        match data {
            Err(err) => {
                embed.color(Color::RED).title("Something went wrong...").description(err);
                embed
            },
            Ok(data) => {
                let data_len = data.len();
                if data_len == 0 {
                    embed.color(Color::RED).description("No kills this week?! Wtf, GGI.");
                } else {
                    let result: Vec<(String, u32)> = get_sorted_weekly_summary(data);
                    let summary = result.into_iter().enumerate().take(10).map(|(i,e)| DataRow {
                        position: u32::try_from(i).unwrap() + 1,
                        cmdr: e.0.to_owned(),
                        kill_count: e.1,
                        highlight: false
                    }).collect::<Vec<_>>();
                    embed.color(Color::DARK_GREEN).description(format!("This week, GGI logged a total of {} kills. Here's the top 10.", data_len));

                    build_weekly_embeds(summary, embed);
                }
                embed
            }
        }
    }) ).await;

    return match message_sent_result {
        Ok(_) => {
            println!("Scheduled Killboard Message sent");
            Ok(())
        },
        Err(err) => {
            println!("Failed to send scheduled killboard message");
            let err_msg = err.to_string();
            println!("{}",&err_msg);
            Err(err_msg)
        }
    };
}

fn get_relevant_time_range_summary() -> (DateTime<Utc>, DateTime<Utc>) {
    let now = chrono::offset::Utc::now();

    let offset_to_end = match now.weekday() {
        chrono::Weekday::Mon => 4,
        chrono::Weekday::Tue => 5,
        chrono::Weekday::Wed => 6,
        chrono::Weekday::Thu => 7,
        chrono::Weekday::Fri => 1,
        chrono::Weekday::Sat => 2,
        chrono::Weekday::Sun => 3,
    };

    let mut start = now.sub(Days::new(offset_to_end));

    fn override_hours_and_minutes_to_0800(date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        return date.with_hour(8)?.with_minute(0)?.with_second(0)?.with_nanosecond(0)
    }

    match override_hours_and_minutes_to_0800(start) {
        Some(data) => start = data,
        None => println!("[WARN] Failed to create correct timestamp for finding relevant range"),
    }

    let end = start.add(Days::new(7));

    return (start, end)
}



pub fn build_weekly_embeds(data: Vec<DataRow>, embed: &mut poise::serenity_prelude::CreateEmbed) {
    embed.color(Color::DARK_GREEN).title("Result for current leaderboard week");

    let mut position_row: Vec<String> = vec![];
    let mut cmdr_row: Vec<String> = vec![];
    let mut count_row: Vec<String> = vec![];

    for entry in data {
        match entry.highlight {
            true => {
                position_row.push(format!("**{}**", entry.position));
                cmdr_row.push(format!("**{}**", entry.cmdr));
                count_row.push(format!("**{}**", entry.kill_count));
            }
            false => {
                position_row.push(format!("{}", entry.position));
                cmdr_row.push(format!("{}", entry.cmdr));
                count_row.push(format!("{}", entry.kill_count));
            }
        }
    }

    embed.field(":trophy:", position_row.join("\n"), true)
        .field(":busts_in_silhouette:", cmdr_row.join("\n"), true)
        .field("×:dagger:", count_row.join("\n"), true);
}


pub fn is_cmdr_in_result_set(data: &Vec<(String, u32)>, cmdr: &String) -> bool {
    return data.iter().find(|&x| x.0.eq_ignore_ascii_case(&cmdr)).is_some();
}
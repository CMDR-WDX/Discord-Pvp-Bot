use std::{time::{Duration, UNIX_EPOCH}};


use chrono::{DateTime, Utc};
use poise::serenity_prelude::{CreateEmbed, Color};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use super::super::{Context, Error};

#[derive(Debug, Serialize, Deserialize)]
enum HistoryEventType {
    #[serde(rename="died")]
    Died,
    #[serde(rename="killed")]
    Killed
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryEntry {
    #[serde(rename="eventType")]
    event_type: HistoryEventType,
    #[serde(rename="otherCmdr")]
    other_cmdr: String,
    timestamp: u64
}

impl HistoryEntry {
    pub fn as_string_line(self) -> String {
        let parsed_timestamp = DateTime::<Utc>::from(UNIX_EPOCH+Duration::from_secs(self.timestamp));
        let timestamp_string = parsed_timestamp.format("%Y-%m-%d %H:%M").to_string();

        let start_emoji = match self.event_type {
            HistoryEventType::Killed => ":dagger:",
            HistoryEventType::Died => ":skull:"
        };

        return format!("{} **CMDR {}** | {}", start_emoji, self.other_cmdr, timestamp_string);
    }
}


impl HistoryEntry {
    pub fn new(event_type: HistoryEventType, other_cmdr: String, timestamp: u64) -> Self {
        HistoryEntry { event_type, other_cmdr, timestamp }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CmdrWhoisLookupResponseSuccess {
    #[serde(rename="cmdrName")]
    cmdr_name: String,
    kills: u32,
    deaths: u32,
    #[serde(rename="recentHistory")]
    recent_history: Vec<HistoryEntry>
}

impl CmdrWhoisLookupResponseSuccess {
    pub fn new(cmdr_name: String, kills: u32, deaths: u32, recent_history: Vec<HistoryEntry>) -> Self {
        return CmdrWhoisLookupResponseSuccess { cmdr_name, kills, deaths, recent_history }
    }
    fn mock_data(cmdr_name: String) -> Self {
        let entries = vec![
            HistoryEntry::new(HistoryEventType::Died, "WDX".to_string(), 1671647068),
            HistoryEntry::new(HistoryEventType::Killed, "Banana".to_string(), 1671644068),
            HistoryEntry::new(HistoryEventType::Killed, "Test123".to_string(), 1671643068),
        ];
        return CmdrWhoisLookupResponseSuccess::new(cmdr_name, 214, 31, entries);
    }

    fn apply_to_embed(self, embed: &mut CreateEmbed, callee: String) -> &mut CreateEmbed {
        embed.title(format!("CMDR {}", self.cmdr_name));
        embed.footer(|f| f.text(format!("Asked by {}", callee)));
        let history_string = self.recent_history.into_iter()
            .map(|e| e.as_string_line())
            .fold(String::new(), |mut a, b| {
                a.reserve(b.len()+1);
                a.push_str(b.as_str());
                a.push_str("\n");
                a
            });
        let history_string = history_string.trim_end();
        embed.description(format!(":dagger: × {}        :skull: × {}\n\n__Recent Activity__\n\n{}", self.kills, self.deaths,history_string));
    
        return embed;
    }

    pub async fn get_from_server(cmdr_name: String) -> Result<Option<Self>, String> {
        let safe_cmdr_name = urlencoding::encode(cmdr_name.as_str()).into_owned();
        let server_url = std::env::var("DATA_SERVER_ADDRESS").map_err(|_| "Bot misconfigured. DATA_SERVER_ADRESS not defined.")?;
        let server_auth = std::env::var("DATA_SERVER_AUTH").map_err(|_| "Bot misconfigured. DATA_SERVER_AUTH not defined.")?;

        async fn run_request(path: String, server_auth: String) -> Result<Option<CmdrWhoisLookupResponseSuccess>, reqwest::Error> {
            let response = reqwest::Client::new()
            .get(path)
            .bearer_auth(server_auth)
            .send()
            .await;

            match response {
                Ok(response_ok) => {
                    if response_ok.status() == StatusCode::NOT_FOUND {
                        return Ok(None);
                    }

                    let parsed_response: CmdrWhoisLookupResponseSuccess = 
                    response_ok
                        .json()
                        .await?;
                    return Ok(Some(parsed_response));
                },
                Err(bad_response) => {
                    return Err(bad_response)
                }
            }

        }
        
        let response = run_request(format!("{}/api/user/{}", server_url, safe_cmdr_name), server_auth).await;
        match response {
            Ok(good_response) => Ok(good_response),
            Err(bad_response) => Err(bad_response.to_string())
        }
    }
}


#[poise::command(slash_command)]
pub async fn pvpwhois(
    ctx: Context<'_>,
    #[description="The CMDR name (without the CMDR Prefix)"] cmdr: String
) -> Result<(), Error> {

    let response =  CmdrWhoisLookupResponseSuccess::get_from_server(cmdr).await;


    let message = ctx.send(|builder| {
        builder
            .embed(|embed| {
                match response {
                    Err(err) => {
                        embed.color(Color::RED).title("Error").description(err)
                    },
                    Ok(data) => {
                        match data {
                            Some(response) => {
                                response.apply_to_embed(embed, ctx.author().tag()).color(Color::DARK_GREEN)
                            },
                            None => {
                                embed.color(Color::GOLD).title("Not Found").description("We do not have any data for this CMDR.")
                            }
                        }
                    }
                }
            })
    }).await;

    if let Err(reason) = message {
        println!("Error Sending Message: {:?}", reason)
    }

    Ok(())
}
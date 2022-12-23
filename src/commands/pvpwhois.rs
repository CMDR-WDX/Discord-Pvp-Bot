use poise::serenity_prelude::{CreateEmbed, Color};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use super::super::{Context, Error};

#[derive(Debug, Serialize, Deserialize)]
enum HistoryEventType {
    Died,
    Killed
}


struct HistoryEntry {
    event_type: HistoryEventType,
    other_cmdr: String,
    timestamp: String,
    location: Option<String>
}

impl HistoryEntry {
    fn from_json(data: &HistoryEntryJson, self_cmdr: String) -> Option<Self> {
        let event_type: HistoryEventType;
        if self_cmdr.eq_ignore_ascii_case(&data.killer_name) {
            event_type = HistoryEventType::Killed;
        } else if self_cmdr.eq_ignore_ascii_case(&data.victim_name) {
            event_type = HistoryEventType::Died;
        } else {
            return None;
        }
        let event_type = event_type;
        let other_cmdr = match event_type {
            HistoryEventType::Killed => data.victim_name.clone(),
            HistoryEventType::Died => data.killer_name.clone()
        };
        let location: Option<String>;
        match data.location.clone() {
            None => location = None,
            Some(val) => {
                if val.to_lowercase().contains("unknown") {
                    location = None;
                }
                else {
                    location = Some(val);
                }
            }
        }
        return Some(HistoryEntry::new(event_type, other_cmdr, data.timestamp.clone(), location));
    }

    pub fn new(event_type: HistoryEventType, other_cmdr: String, timestamp: String, location: Option<String>) -> Self {
        HistoryEntry { event_type, other_cmdr, timestamp, location }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct HistoryEntryJson {
    timestamp: String,
    killer_name: String,
    killer_ship: String,
    killer_rank: String,
    victim_name: String,
    victim_ship: String,
    victim_rank: String,
    location: Option<String>
}



#[derive(Debug, Serialize, Deserialize)]
struct CmdrWhoisLookupResponseSuccess {
    #[serde(rename="cmdrName")]
    cmdr_name: String,
    kills: u32,
    deaths: u32,
    #[serde(rename="recentHistory")]
    recent_history: Vec<HistoryEntryJson>
}

impl CmdrWhoisLookupResponseSuccess {


    fn apply_to_embed(self, embed: &mut CreateEmbed, callee: String) -> &mut CreateEmbed {
        let historic_data: Vec<HistoryEntry> = self.recent_history.iter().filter_map(|f| HistoryEntry::from_json(f, self.cmdr_name.clone())).collect();
        let historic_data_len = historic_data.len();
        let mut cmdr_row: Vec<String> = vec![];
        let mut system_row: Vec<String> = vec![];
        let mut date_row: Vec<String> = vec![];

        let mut has_system_data = false;

        for entry in historic_data {
            let emoji_to_use = match entry.event_type {
                HistoryEventType::Killed => ":dagger:",
                HistoryEventType::Died => ":skull:"
            };
            cmdr_row.push(format!("{} {}", emoji_to_use, entry.other_cmdr));
            let location_string = match entry.location {
                Some(location) => {
                    has_system_data = true;
                    location
                },
                None => "".to_string()
            };
            system_row.push(location_string);
            date_row.push(entry.timestamp);
            //system_row.push(location_string);
            //timestamp_row.push(format!("{}\n", entry.timestamp));
        }

        let description_as_heading = match historic_data_len {
            0 => "",
            _ => "\n\n**__Recent History__**\n\n"
        };
        embed.title(format!("CMDR {}", self.cmdr_name));
        embed.description(format!(" :dagger: × {} :skull: × {} {}", self.kills, self.deaths, description_as_heading));
        if historic_data_len > 0 {
            embed.field(":busts_in_silhouette:", cmdr_row.join("\n"), true);
            if has_system_data {
                embed.field(":ringed_planet:", system_row.join("\n"), true);
            }
            embed.field(":calendar_spiral:", date_row.join("\n"), true);
        }
        embed.footer(|f| f.text(format!("Asked by {}", callee)));
    
        return embed;
    }

    pub async fn get_from_server(cmdr_name: String) -> Result<Option<Self>, String> {
        let safe_cmdr_name = urlencoding::encode(cmdr_name.as_str()).into_owned();
        let server_url = crate::data::Environment::server_address();
        let server_auth = crate::data::Environment::server_auth();

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
                Err(bad_response) => return Err(bad_response)
            }

        }
        
        let response = run_request(format!("{}/api/bot/user/{}", server_url, safe_cmdr_name), server_auth).await;
        match response {
            Ok(good_response) => Ok(good_response),
            Err(bad_response) => Err(bad_response.to_string())
        }
    }
}

/// Look up a CMDR on the bot.
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
                            Some(response) => response.apply_to_embed(embed, ctx.author().tag()).color(Color::DARK_GREEN),
                            None => embed.color(Color::GOLD).title("Not Found").description("We do not have any data for this CMDR.")
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
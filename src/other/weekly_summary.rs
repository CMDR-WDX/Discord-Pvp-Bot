use std::{ops::{Add, Sub}, collections::HashMap};
use chrono::{Datelike, Timelike, DateTime, Utc, Days};
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};

pub async fn get_kills_for_current_cycle() -> Result<Vec<RangeResponseEntry>, String> {
    let (start, end) = get_relevant_time_range();

    let result_with_potentially_out_of_bounds_events =  fetch_from_server(get_query_string_for_api_call(start, end)).await?;
    
    let filtered_result: Vec<RangeResponseEntry> = result_with_potentially_out_of_bounds_events.into_iter()
        .filter(|x| x.timestamp.ge(&start) && x.timestamp.le(&end)).collect::<Vec<_>>();


    return Ok(filtered_result);
}
/// The API expects a lower and upper date in YYYY-MM-DD Format.
/// Note that the lower bound is inclusive, the upper bound exclusive
/// This means a search for 2023-01-01 - 2023-01-10 will have all kills between the *start* of 2023-01-01 and the *start* of 2023-01-10.
/// We also want all kills till the *end* of 2023-01-10. To get this, we basically just add a day at the end.
pub fn get_query_string_for_api_call(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    let start_string = start.format("%Y-%m-%d");
    let end_string = end.add(Days::new(1)).format("%Y-%m-%d");

    return format!("?from={}&to={}", start_string, end_string)

}

#[derive(Serialize, Debug, Deserialize)]
struct GetRangeResponse {
    success: String,
    count: u32,
    kills: Vec<_RangeResponseEntry>
}

#[derive(Serialize, Debug, Deserialize)]
struct _RangeResponseEntry {
    id: u64,
    timestamp: String,
    killer_name: String,
    victim_name: String,
    location: String
}

#[derive(Serialize, Debug, Deserialize)]
pub struct RangeResponseEntry {
    id: u64,
    timestamp: DateTime<Utc>,
    killer_name: String,
    victim_name: String,
    location: Option<String>
}

impl _RangeResponseEntry {
    fn convert(self: Self) -> RangeResponseEntry {

        let id = self.id;
        let timestamp: DateTime<Utc> = match DateTime::parse_from_str(format!("{} +00:00", &self.timestamp).as_str(), "%Y-%m-%d %H:%M:%S %z") {
            Ok(data) => data.with_timezone(&Utc),
            Err(err) => {
                println!("[ERROR]: Failed to parse Timestamp. Using Utc.now as fallback. Error below:\n{err}");
                Utc::now()
            },
        }; 
        let location: Option<String> = match self.location.eq_ignore_ascii_case("UNKNOWN") {
            true => None,
            false => Some(self.location)
        };
        let killer_name = self.killer_name;
        let victim_name = self.victim_name;
        return RangeResponseEntry {
            id,
            timestamp,
            killer_name,
            victim_name,
            location
        }
    }
}


pub async fn fetch_from_server(query_string: String) -> Result<Vec<RangeResponseEntry>, String> {
    let server_url = crate::data::Environment::server_address();
    let server_auth = crate::data::Environment::server_auth();


    async fn run_request(path: String, server_auth: String) -> Result<Option<Vec<_RangeResponseEntry>>, reqwest::Error> {
        let response = reqwest::Client::new()
            .get(path).bearer_auth(server_auth).send().await?;

        if response.status() != StatusCode::OK {
            return Ok(None);
        }
        let parsed_response: GetRangeResponse = response.json::<GetRangeResponse>().await?;
        return Ok(Some(parsed_response.kills))
    }

    match run_request(format!("{server_url}/killboard/get/kills{query_string}"), server_auth).await {
        Ok(data) => match data {
            Some(e) => {
                let mapped_data = e.into_iter().map(|x| x.convert()).collect::<Vec<_>>();
                Ok(mapped_data)
            },
            None => Err("Failed to receive any data from Range Query".to_string())
        },
        Err(err) => Err(err.to_string()),
    }

}

pub fn get_relevant_time_range() -> (DateTime<Utc>, DateTime<Utc>) {
    let now = chrono::offset::Utc::now();

    let offset_to_end = match now.weekday() {
        chrono::Weekday::Mon => 4,
        chrono::Weekday::Tue => 5,
        chrono::Weekday::Wed => 6,
        chrono::Weekday::Thu => 0,
        chrono::Weekday::Fri => 1,
        chrono::Weekday::Sat => 2,
        chrono::Weekday::Sun => 3,
    };

    let mut start = if offset_to_end == 0 && now.hour() < 8 {
        now.sub(Days::new(7))
    } else { 
        now.sub(Days::new(offset_to_end))
    };

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

pub fn get_sorted_weekly_summary(data: Vec<RangeResponseEntry>) -> Vec<(String, u32)> {
    let mut summary_map: HashMap<String, u32> = HashMap::new();

    data.iter().for_each(|x| {
        if !summary_map.contains_key(&x.killer_name) {
            summary_map.insert(x.killer_name.to_owned(), 0);
        }
        summary_map.insert(x.killer_name.to_owned(), summary_map.get(&x.killer_name).unwrap()+1);
    });

    let mut as_tuple_vec: Vec<(String, u32)>  =  summary_map.iter().map(|(k,v)| { (k.to_owned(),v.to_owned())}).collect::<_>();
    as_tuple_vec.sort_by(|a,b| {b.1.cmp(&a.1) });
    as_tuple_vec
}
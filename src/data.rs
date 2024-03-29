
use once_cell::sync::OnceCell;
use tokio_cron_scheduler::Job;



#[derive(Clone, Debug)]
pub struct Environment {
    discord_token: String,
    ggi_guild_id: u64,
    data_server_address: String,
    data_server_auth: String,
    role_authenticated: u64,
    role_administrator: u64,
    discord_weekly_update_channel_id: u64,
    weekly_summary_cron: String
}

static ENVIRONMENT: OnceCell<Environment> = OnceCell::new();



impl Environment {

    pub fn guild_id() -> u64 {
        return ENVIRONMENT.get().unwrap().ggi_guild_id;
    }

    pub(crate) fn discord_token() -> String {
        return ENVIRONMENT.get().unwrap().discord_token.clone();
    }

    pub fn role_admin() -> u64 {
        return ENVIRONMENT.get().unwrap().role_administrator;
    }

    pub fn role_auth() -> u64 {
        return ENVIRONMENT.get().unwrap().role_authenticated;
    }

    pub fn discord_weekly_update_channel_id() -> u64 {
        return ENVIRONMENT.get().unwrap().discord_weekly_update_channel_id;
    }

    pub fn server_auth() -> String {
        return ENVIRONMENT.get().unwrap().data_server_auth.clone()
    }

    pub fn server_address() -> String {
        return ENVIRONMENT.get().unwrap().data_server_address.clone()
    }

    pub fn weekly_summary_cron() -> String {
        return ENVIRONMENT.get().unwrap().weekly_summary_cron.clone()
    }

}


pub fn startup_check() -> Result<(), String> {
    
    if let Err(_) = dotenv::dotenv() {
        println!("[INFO] No .env found. No Environment Variables were read.")
    }

    let required_vars = vec![
        "DISCORD_TOKEN",
        "GGI_GUILD_ID",
        "DATA_SERVER_ADDRESS",
        "DATA_SERVER_AUTH",
        "ROLE_AUTHENTICATED",
        "ROLE_ADMINISTRATOR",
        "DISCORD_WEEKLY_UPDATE_CHANNEL_ID"
    ];
    let missing_vars: Vec<String> = required_vars.iter().filter(|x| std::env::var(x).is_err()).map(|x|x.to_string()).collect();

    if missing_vars.len() > 0 {
        let joined_string = missing_vars.join(",");
        return Err(format!("Cannot starts. The following environment variables must be set: [{}]", joined_string));
    }

    // These vars must be an Integer value
    let integer_vars = vec![
        "GGI_GUILD_ID",
        "ROLE_AUTHENTICATED",
        "ROLE_ADMINISTRATOR",
        "DISCORD_WEEKLY_UPDATE_CHANNEL_ID"
    ];

    let non_integer_vars: Vec<String> = integer_vars.iter().filter(|x| {
        let as_u64_result: Result<u64, _> = std::env::var(x).unwrap().parse();
        return as_u64_result.is_err();
    }).map(|x|x.to_string()).collect();

    if non_integer_vars.len() > 0 {
        let joined_string = non_integer_vars.join(",");
        return Err(format!("Cannot start. The following environ variables cannot be turned into integers: [{}]", joined_string));
    }

    // Get optional override cron job notation for the Weekly Post
    let weekly_summary_cron = match std::env::var("WEEKLY_SUMMARY_CRON_OVERRIDE") {
        Err(err) => match err {
            std::env::VarError::NotPresent => "0 0 8 * * Thu".to_owned(),
            _ => panic!("Failed to parse WEEKLY_SUMMARY_CRON_OVERRIDE"),
        },
        Ok(val) => {
            match Job::new(val.as_str(), |_x, _y|{}) {
                Err(err) => {
                    panic!("Failed to parse WEEKLY_SUMMARY_CRON_OVERRIDE: {}", err);
                },
                Ok(_) => val
            }
        }
    };


    // After no Errors are found, Initialize the Lazy Value.
    let init_env = Environment { 
        discord_token: std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"), 
        ggi_guild_id: std::env::var("GGI_GUILD_ID").expect("missing GGI_GUILD_ID").parse().expect("Failed to parse GGI_GUILD_ID"), 
        discord_weekly_update_channel_id: std::env::var("DISCORD_WEEKLY_UPDATE_CHANNEL_ID").expect("missing DISCORD_WEEKLY_UPDATE_CHANNEL_ID").parse().expect("Failed to parse DISCORD_WEEKLY_UPDATE_CHANNEL_ID"), 
        data_server_address: std::env::var("DATA_SERVER_ADDRESS").expect("missing DATA_SERVER_ADDRESS"), 
        data_server_auth: std::env::var("DATA_SERVER_AUTH").expect("missing DATA_SERVER_AUTH"), 
        role_authenticated: std::env::var("ROLE_AUTHENTICATED").expect("missing ROLE_AUTHENTICATED").parse().expect("Failed to parse ROLE_AUTHENTICATED"),
        role_administrator: std::env::var("ROLE_ADMINISTRATOR").expect("missing ROLE_ADMINISTRATOR").parse().expect("Failed to parse ROLE_ADMINISTRATOR"),
        weekly_summary_cron: weekly_summary_cron
    };

    ENVIRONMENT.set(init_env).unwrap();

    return Ok(());


}
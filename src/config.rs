use dotenv::vars;

pub struct Config {
    pub discord_token: String,
    pub userdata_auth: String,
    pub server_addr: String,
    pub pg: deadpool_postgres::Config,
}
impl Config {
    pub fn new() -> Self {
        let environment_vars: Vec<(String, String)> = vars().collect();
        Config {
            discord_token: find_key(&environment_vars, "DISCORD_TOKEN"),
            userdata_auth: find_key(&environment_vars, "USERDATA_AUTH"),
            server_addr: find_key(&environment_vars, "SERVER_ADDR"),
            pg: deadpool_postgres::Config::new(),
        }
    }
}

pub fn find_key(iteration: &Vec<(String, String)>, key_search: &'static str) -> String {
    match iteration.iter().find(|(key, _)| key == key_search) {
        Some((_, value)) => value.to_string(),
        None => panic!(
            "couldn't find '{}' in the environment variables",
            key_search
        ),
    }
}

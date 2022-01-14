use crate::errors::MyError;
use crate::config::Config;
use crate::models::UserData;
use crate::constants::{roles, persistent_roles, C2SGUILD};
use twilight_http::Client;
use twilight_model::id::RoleId;
use twilight_model::{id::{GuildId, UserId},guild::Member};

pub async fn handle_roles(user_data: UserData, config: Config) -> Result<Vec<&'static str>, MyError> {
  let mut gained_roles: Vec<&'static str> = Vec::new();
  let client = Client::new(config.discord_token);
  let guild_id = GuildId::new(C2SGUILD).ok_or(MyError::NotFound)?;
  let user_id = UserId::new(match str::parse::<u64>(&user_data.discordId) {
    Ok(value) => value,
    Err(_) => return Err(MyError::NotFound),
  }).ok_or(MyError::NotFound)?;

  let member_data = match client
    .guild_member(guild_id, user_id)
    .exec().await {
      Ok(value) => value,
      Err(_) => return Err(MyError::NotFound),
    };
  let member_data: Member = match member_data.model().await {
    Ok(value) => value,
    Err(_) => return Err(MyError::NotFound),
  };

  let mut gained_metabit_roles = handle_metabit_roles(&mut gained_roles, &member_data, &user_data);
  let mut gained_paleo_roles = handle_paleo_roles(&mut gained_roles, &member_data, &user_data);
  let mut gained_simulation_roles = handle_simulation_roles(&mut gained_roles, &member_data, &user_data);

  let mut applyable_roles = persistent_roles::PERSISTENT_ROLES
    .into_iter()
    .filter_map(RoleId::new)
    .filter(|role| member_data.roles.contains(role))
    .collect::<Vec<RoleId>>();

  applyable_roles.append(&mut gained_metabit_roles);
  applyable_roles.append(&mut gained_paleo_roles);
  applyable_roles.append(&mut gained_simulation_roles);
  
  let updated_member_data = match client.update_guild_member(guild_id, user_id)
    .roles(applyable_roles.as_slice())
    .exec()
    .await {
      Ok(value) => value,
      Err(_) => return Err(MyError::NotFound),
    };
  match updated_member_data
  .model()
  .await {
    Ok(value) => value,
    Err(_) => return Err(MyError::NotFound),
  };

  Ok(gained_roles)
}

fn handle_metabit_roles(roles: &Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  vec![1]
  .into_iter()
  .filter_map(RoleId::new)
  .collect::<Vec<RoleId>>()
}

fn handle_paleo_roles(roles: &Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  vec![1]
  .into_iter()
  .filter_map(RoleId::new)
  .collect::<Vec<RoleId>>()
}

fn handle_simulation_roles(roles: &Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  vec![1]
  .into_iter()
  .filter_map(RoleId::new)
  .collect::<Vec<RoleId>>()
}

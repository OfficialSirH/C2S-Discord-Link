use crate::errors::MyError;
use crate::config::Config;
use crate::models::UserData;
use crate::constants::{roles, persistent_roles, C2SGUILD};
use twilight_http::Client;
use twilight_model::id::RoleId;
use twilight_model::{id::{GuildId, UserId},guild::Member};

trait GetSomeFromRoleId {
  fn get_some(self: Self) -> RoleId;
}

impl GetSomeFromRoleId for Option<RoleId> {
  fn get_some(self: Self) -> RoleId {
      match self {
        Some(role_id) => role_id,
        None => unreachable!()
      }
  }
}

pub async fn handle_roles(user_data: UserData, config: Config) -> Result<Vec<&'static str>, MyError> {
  let mut gained_roles: Vec<&'static str> = Vec::new();
  let client = Client::new(config.discord_token);
  let guild_id = GuildId::new(C2SGUILD).ok_or(MyError::NotFound)?;
  let user_id = UserId::new(match str::parse::<u64>(&user_data.discord_id) {
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

fn handle_metabit_roles(roles: &mut Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  // the length of the vector will always be 1 to simplify the process of combining the roles array
  let mut applyable_roles: Vec<RoleId> = Vec::new();

  if user_data.metabits >= 100e12 {
    let role = RoleId::new(roles::REALITY_LEGEND).get_some();
    if member.roles.contains(&role) {
      roles.push("Reality Legend");
    }
    applyable_roles.push(role);
  }

  else if user_data.metabits >= 1e12 {
    let role = RoleId::new(roles::REALITY_EXPERT).get_some();
    if member.roles.contains(&role) {
      roles.push("Reality Expert");
    }
    applyable_roles.push(role);
  }

  else if user_data.metabits >= 1e9 {
    let role = RoleId::new(roles::REALITY_EXPLORER).get_some();
    if member.roles.contains(&role) {
      roles.push("Reality Explorer");
    }
    applyable_roles.push(role);
  }

  return applyable_roles;
}

fn handle_paleo_roles(roles: &mut Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  // the length of the vector will always be 1 to simplify the process of combining the roles array
  let mut applyable_roles: Vec<RoleId> = Vec::new();
  let dino_prestige = (user_data.dino_rank - 50) % 50;

  if dino_prestige == 10 {
    let role = RoleId::new(roles::PALEONTOLOGIST_LEGEND).get_some();
    if member.roles.contains(&role) {
      roles.push("Paleontologist Legend");
    }
    applyable_roles.push(role);
  }

  else if dino_prestige == 1 {
    let role = RoleId::new(roles::PROGRESSIVE_PALEONTOLOGIST).get_some();
    if member.roles.contains(&role) {
      roles.push("Progressive Paleontologist");
    }
    applyable_roles.push(role);
  }

  else if user_data.dino_rank >= 26 {
    let role = RoleId::new(roles::PALEONTOLOGIST).get_some();
    if member.roles.contains(&role) {
      roles.push("Paleontologist");
    }
    applyable_roles.push(role);
  }



  return applyable_roles;
}

fn handle_simulation_roles(roles: &mut Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  // the length of the vector will always be 1 to simplify the process of combining the roles array
  let mut applyable_roles: Vec<RoleId> = Vec::new();

  if user_data.all_hidden_achievements_obtained {
    let role = RoleId::new(roles::FINDER_OF_SEMBLANCE_SECRETS).get_some();
    if member.roles.contains(&role) {
      roles.push("Finder of Semblance's Secrets");
    }
    applyable_roles.push(role);
  }

  else {
    if user_data.singularity_speedrun_time <= 120.0 {
      let role = RoleId::new(roles::SONIC_SPEEDSTER_OF_SIMULATIONS).get_some();
      if member.roles.contains(&role) {
        roles.push("Sonic Speedster of Simulations");
      }
      applyable_roles.push(role);
    }

    else if user_data.singularity_speedrun_time <= 300.0 {
      let role = RoleId::new(roles::SIMULATION_SPEEDSTER).get_some();
      if member.roles.contains(&role) {
        roles.push("Simulation Speedster");
      }
      applyable_roles.push(role);
    }

    if user_data.all_sharks_obtained {
      let role = RoleId::new(roles::SHARK_COLLECTOR).get_some();
      if member.roles.contains(&role) {
        roles.push("Shark Collector");
      }
      applyable_roles.push(role);
    }
  }

  if user_data.beta_tester {
    let role = RoleId::new(roles::BETA_TESTER).get_some();
    if member.roles.contains(&role) {
      roles.push("Beta Tester");
    }
    applyable_roles.push(role);
  }

  return applyable_roles;
}

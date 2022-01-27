use crate::errors::MyError;
use crate::config::Config;
use crate::models::UserData;
use crate::constants::{roles, persistent_roles, C2SGUILD, MetabitRequirements, PaleoRequirements, SimulationRequirements};
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

pub async fn handle_roles(user_data: &UserData, config: Config) -> Result<Vec<&'static str>, MyError> {
  let mut gained_roles: Vec<&'static str> = Vec::new();
  let client = Client::new(config.discord_token);
  let guild_id = GuildId::new(C2SGUILD).ok_or(MyError::InternalError("failed GuildId struct"))?;
  let user_id = UserId::new(match str::parse::<u64>(&user_data.discord_id) {
    Ok(value) => value,
    Err(_) => return Err(MyError::InternalError("parsing discord id failed")),
  }).ok_or(MyError::InternalError("failed UserId struct"))?;
  
  let member_data = match client
    .guild_member(guild_id, user_id)
    .exec().await {
      Ok(value) => value,
      Err(_) => 
        return Err(MyError::InternalError("failed retrieving member data (this usually occurs when you're not in the Discord server)"))
      ,
    };
  let member_data: Member = match member_data.model().await {
    Ok(value) => value,
    Err(_) => return Err(MyError::InternalError("failed at parsing the member data to a Member struct")),
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
      Err(_) => return Err(MyError::InternalError("failed at updating member roles")),
    };
  match updated_member_data
  .model()
  .await {
    Ok(value) => value,
    Err(_) => return Err(MyError::InternalError("failed at parsing member data model")),
  };

  Ok(gained_roles)
}

fn handle_metabit_roles(gained_roles: &mut Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  // the length of the vector will always be 1 to simplify the process of combining the roles array
  let mut applyable_roles: Vec<RoleId> = Vec::new();

  if user_data.metabits >= MetabitRequirements::RealityLegend as i64 {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::REALITY_LEGEND, "Reality Legend"));
  }

  else if user_data.metabits >= MetabitRequirements::RealityExpert as i64 {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::REALITY_EXPERT, "Reality Expert"));
  }

  else if user_data.metabits >= MetabitRequirements::RealityExplorer as i64 {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::REALITY_EXPLORER, "Reality Explorer"));
  }

  return applyable_roles;
}

fn handle_paleo_roles(gained_roles: &mut Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  // the length of the vector will always be 1 to simplify the process of combining the roles array
  let mut applyable_roles: Vec<RoleId> = Vec::new();
  let dino_prestige = (user_data.dino_rank / 50).clamp(0, 10);

  if dino_prestige == PaleoRequirements::PaleontologistLegend as i32 {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::PALEONTOLOGIST_LEGEND, "Paleontologist Legend"));
  }

  else if dino_prestige == PaleoRequirements::ProgressivePaleontologist as i32 {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::PROGRESSIVE_PALEONTOLOGIST, "Progressive Paleontologist"));
  }

  else if user_data.dino_rank >= PaleoRequirements::Paleontologist as i32 {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::PALEONTOLOGIST, "Paleontologist"));
  }



  return applyable_roles;
}

fn handle_simulation_roles(gained_roles: &mut Vec<&'static str>, member: &Member, user_data: &UserData) -> Vec<RoleId> {
  // the length of the vector will always be 1 to simplify the process of combining the roles array
  let mut applyable_roles: Vec<RoleId> = Vec::new();

  if user_data.all_hidden_achievements_obtained {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::FINDER_OF_SEMBLANCE_SECRETS, "Finder of Semblance's Secrets"));
  }

  else {
    if user_data.singularity_speedrun_time <= SimulationRequirements::SonicSpeedsterOfSimulations as i32 as f64 {
      applyable_roles.push(apply_a_role(gained_roles, member, roles::SONIC_SPEEDSTER_OF_SIMULATIONS, "Sonic Speedster of Simulations"));
    }

    else if user_data.singularity_speedrun_time <= SimulationRequirements::SimulationSpeedster as i32 as f64 {
      applyable_roles.push(apply_a_role(gained_roles, member, roles::SIMULATION_SPEEDSTER, "Simulation Speedster"));
    }

    if user_data.all_sharks_obtained {
      applyable_roles.push(apply_a_role(gained_roles, member, roles::SHARK_COLLECTOR, "Shark Collector"));
    }
  }

  if user_data.beta_tester {
    applyable_roles.push(apply_a_role(gained_roles, member, roles::BETA_TESTER, "Beta Tester"));
  }

  return applyable_roles;
}

fn apply_a_role(gained_roles: &mut Vec<&'static str>, member: &Member, role_id: u64, role_name: &'static str) -> RoleId {
  let role = RoleId::new(role_id).get_some();
  if !member.roles.contains(&role) {
    gained_roles.push(role_name);
  }
  role
}

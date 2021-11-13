import type { Snowflake } from 'discord.js';
import type { GuildMember } from 'discord.js';
import type { IBody } from '../../server';
import { CONSTANTS } from '../../config';
const {
  PALEONTOLOGIST_LEGEND,
  FINDER_OF_SEMBLANCES_SECRETS,
  SHARK_COLLECTOR,
  SONIC_SPEEDSTER_OF_SIMULATIONS,
  REALITY_LEGEND,
  PROGRESSIVE_PALEONTOLOGIST,
  REALITY_EXPERT,
  PALEONTOLOGIST,
  SIMULATION_SPEEDSTER,
  REALITY_EXPLORER,
  BETA_TESTER,
} = CONSTANTS.ROLES;

export async function handleRoles(data: IBody, member: GuildMember): Promise<string[]> {
  const gainedRoles: Snowflake[] = [];
  await member.roles.set([
    ...Object.values(CONSTANTS.PERSISTENT_ROLES).filter(role => member.roles.cache.has(role)),
    ...handleMetabitRoles(data, member, gainedRoles),
    ...handlePaleoRoles(data, member, gainedRoles),
    ...handleSimulationRoles(data, member, gainedRoles),
  ]);
  return gainedRoles;
}

function handleMetabitRoles(data: IBody, member: GuildMember, gainedRoles: string[]): Snowflake[] {
  const applyableRoles: Snowflake[] = [],
    cache = member.roles.cache;
  if (data.metabits >= 100e12) {
    applyableRoles.push(REALITY_LEGEND);
    if (!cache.has(REALITY_LEGEND)) gainedRoles.push('Reality Legend');
  } else if (data.metabits >= 1e12) {
    applyableRoles.push(REALITY_EXPERT);
    if (!cache.has(REALITY_EXPERT)) gainedRoles.push('Reality Expert');
  } else if (data.metabits >= 1e9) {
    applyableRoles.push(REALITY_EXPLORER);
    if (!cache.has(REALITY_EXPLORER)) gainedRoles.push('Reality Explorer');
  }
  return applyableRoles;
}

function handlePaleoRoles(data: IBody, member: GuildMember, gainedRoles: string[]): Snowflake[] {
  const applyableRoles: Snowflake[] = [],
    dino_prestige = Math.floor((data.dino_rank - 50) / 50),
    cache = member.roles.cache;
  if (dino_prestige == 10) {
    applyableRoles.push(PALEONTOLOGIST_LEGEND);
    if (!cache.has(PALEONTOLOGIST_LEGEND)) gainedRoles.push('Paleontologist Legend');
  } else if (dino_prestige == 1) {
    applyableRoles.push(PROGRESSIVE_PALEONTOLOGIST);
    if (!cache.has(PROGRESSIVE_PALEONTOLOGIST)) gainedRoles.push('Progressive Paleontologist');
  } else if (data.dino_rank >= 26) {
    applyableRoles.push(PALEONTOLOGIST);
    if (!cache.has(PALEONTOLOGIST)) gainedRoles.push('Paleontologist');
  }
  return applyableRoles;
}

function handleSimulationRoles(data: IBody, member: GuildMember, gainedRoles: string[]): Snowflake[] {
  const applyableRoles: Snowflake[] = [],
    cache = member.roles.cache;
  if (data.all_hidden_achievements_obtained) {
    applyableRoles.push(FINDER_OF_SEMBLANCES_SECRETS);
    if (!cache.has(FINDER_OF_SEMBLANCES_SECRETS)) gainedRoles.push("Finder of Semblance's Secrets");
    applyableRoles.push(SONIC_SPEEDSTER_OF_SIMULATIONS);
    if (!cache.has(SONIC_SPEEDSTER_OF_SIMULATIONS)) gainedRoles.push('Sonic Speedster of Simulations');
    applyableRoles.push(SHARK_COLLECTOR);
    if (!cache.has(SHARK_COLLECTOR)) gainedRoles.push('Shark Collector');
  } else {
    if (data.singularity_speedrun_time <= 120) {
      applyableRoles.push(SONIC_SPEEDSTER_OF_SIMULATIONS);
      if (!cache.has(SONIC_SPEEDSTER_OF_SIMULATIONS)) gainedRoles.push('Sonic Speedster of Simulations');
    } else if (data.singularity_speedrun_time <= 300) {
      applyableRoles.push(SIMULATION_SPEEDSTER);
      if (!cache.has(SONIC_SPEEDSTER_OF_SIMULATIONS)) gainedRoles.push('Simulation Speedster');
    }

    if (data.all_sharks_obtained) {
      applyableRoles.push(SHARK_COLLECTOR);
      if (!cache.has(SHARK_COLLECTOR)) gainedRoles.push('Shark Collector');
    }
  }
  if (data.betaTester) {
    applyableRoles.push(BETA_TESTER);
    if (!cache.has(BETA_TESTER)) gainedRoles.push('Beta Tester');
  }
  return applyableRoles;
}

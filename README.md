# C2S-Discord-Link

the REST API for handling C2S UserData in Cell to Singularity

## Prerequisites

- ### Base URL
  `http://127.0.0.1:3000/userdata`
- ### Authorization
  `Basic base64(email:playertoken)`
- ### UserData Definition

```ts
interface UserData {
  discord_id: Snowflake;
  token: string;
  beta_tester: boolean;
  metabits: number;
  dino_rank: number;
  prestige_rank: number;
  singularity_speedrun_time: number;
  all_sharks_obtained: boolean;
  all_hidden_achievements_obtained: boolean;
  edited_timestamp: number;
}
```

INSERT "UserData" (
    "token",
    "beta_tester",
    "discord_id",
    "metabits",
    "dino_rank",
    "prestige_rank",
    "singularity_speedrun_time",
    "all_sharks_obtained",
    "all_hidden_achievements_obtained",
    "edited_timestamp"
  )
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
RETURNING *;
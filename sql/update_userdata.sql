UPDATE "UserData"
SET "betaTester" = $1,
    "metabits" = $2,
    "dino_rank" = $3,
    "prestige_rank" = $4,
    "singularity_speedrun_time" = $5,
    "all_sharks_obtained" = $6,
    "all_hidden_achievements_obtained" = $7,
    "edited_timestamp" = $8
WHERE "token" = $token
RETURNING *;
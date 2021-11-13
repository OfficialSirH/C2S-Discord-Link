# RESTful-C2SUserData
A repository for storing the REST API for C2S UserData

## Prerequisites 
* ### Base URL
`https://IP:3000/userdata`
* ### Authorization
`process.env.USERDATA_AUTH`
* ### UserData Definition
```ts
interface UserData {
  discordId: Snowflake;
  token: string;
  betaTester: boolean;
  metabits: number;
  dino_rank: number;
  prestige_rank: number;
  singularity_speedrun_time: number;
  all_sharks_obtained: boolean;
  all_hidden_achievements_obtained: boolean;
  edited_timestamp: number;
}
```

## PATH - `/`
* ### **GET** list all entries
Response -
```js
UserData[]
```
* ### **POST** create entry
Body -
```js
{
  discordId: String,
  playerId: String,
  playerToken: String,
  metabits: Number
}
```
Response - 
```js
UserData
```

## PATH - `/{playerId}`
* ### **GET** list entry
Body -
```js
{
  playerToken: String
}
```
Response - 
```js
UserData
```
* ### **POST** update entry
Body - 
```js
{
  playerToken: String,
  metabits: Number
}
```
Response -
```js
UserData
```
* ### **DELETE** delete entry
Body - 
```js
{
  playerToken: String
}
```
Response -
```js
{
  message: 'User Data successfully deleted',
  discordId: String
}
```

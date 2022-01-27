use twilight_model::id::WebhookId;
use twilight_http::{Client,Error};
use crate::{config::Config, constants::{BACKGROUND, LOG, self}};

pub async fn webhook_log(content: String, log_type: LOG) -> Result<(), Error> {
  let config = Config::new();
  let client = Client::new(config.discord_token);
  let webhook_id = unsafe {
    WebhookId::new_unchecked(config.webhook_id.parse::<u64>().unwrap())
  };

  let color = match log_type {
    LOG::SUCCESSFUL => constants::SUCCESSFUL,
    LOG::INFORMATIONAL => constants::INFORMATIONAL,
    LOG::FAILURE => constants::FAILURE,
  };

  match client.execute_webhook(webhook_id, &config.webhook_token)
    .content(format!("```ansi\n{}{}```", BACKGROUND, 
      content.split(' ').map(|word| { format!("{}{}", color, word) }).collect::<Vec<String>>().join(" ")).as_str()
    )
    .exec().await {
      Ok(value) => value,
      Err(error) => return Err(error),
    };

  Ok(())
}

#[tokio::test]
async fn uwu_log() -> Result<(), Error> {
  Ok(webhook_log("UwU, this logger is working! OwO".to_string(), crate::constants::LOG::INFORMATIONAL).await?)
}

#[tokio::test]
async fn failure_log() -> Result<(), Error> {
  Ok(webhook_log("SOMETHING FAILED, OMG!!! RED ALERT, RED ALERT!! WOO WOO WOO WOO!".to_string(), crate::constants::LOG::FAILURE).await?)
}

#[tokio::test]
async fn successful_log() -> Result<(), Error> {
  Ok(webhook_log("YAY! IT WORKED! IT WAS SUCCESSFUL!".to_string(), crate::constants::LOG::SUCCESSFUL).await?)
}

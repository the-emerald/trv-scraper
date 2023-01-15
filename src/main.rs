use anyhow::Context;
use anyhow::Result;
use sea_orm::Database;
use std::env;
use task::fighter::ChampionTask;
use tracing::warn;

pub mod task;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let alchemy_api_key = env::var("ALCHEMY_API_KEY").context("ALCHEMY_API_KEY not set")?;
    let database = Database::connect(db_url).await?;

    let client = reqwest::Client::new();

    let _ = tokio::task::spawn(ChampionTask::new(client, database, alchemy_api_key).run());

    match tokio::signal::ctrl_c().await {
        Ok(()) => Ok(()),
        Err(e) => {
            warn!(e = ?e, "unable to listen to shutdown signal");
            Ok(())
        }
    }
}

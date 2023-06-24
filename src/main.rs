use anyhow::Context;
use anyhow::Result;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use std::env;
use std::time::Duration;
use task::fighter::ChampionTask;
use task::tournament::TournamentTask;

pub mod task;

const CONCURRENT_REQUESTS: usize = 128;

/// 2 hours
const SCRAPE_INTERVAL: u64 = 2 * 60 * 60 * 1000;

const TOURNAMENT_PAGE_SIZE: u64 = 128;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let opt = ConnectOptions::new(env::var("DATABASE_URL").context("DATABASE_URL not set")?)
        .sqlx_logging(false)
        .to_owned();
    let alchemy_api_key = env::var("ALCHEMY_API_KEY").context("ALCHEMY_API_KEY not set")?;
    let database = Database::connect(opt).await?;

    let client = reqwest::Client::new();

    let mut interval = tokio::time::interval(Duration::from_millis(SCRAPE_INTERVAL));
    let champion_task = ChampionTask::new(client.clone(), database.clone(), alchemy_api_key);
    let tournament_task = TournamentTask::new(client, database, TOURNAMENT_PAGE_SIZE);

    loop {
        interval.tick().await;
        // Rescan all champions
        let _ = champion_task.scan().await;

        // Fetch new tournaments
        let _ = tournament_task.scan().await;
    }
}

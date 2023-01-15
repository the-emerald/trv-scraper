use anyhow::Context;
use anyhow::Result;
use sea_orm::Database;
use std::env;
use task::fighter::ChampionTask;

pub mod task;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let alchemy_api_key = env::var("ALCHEMY_API_KEY").context("ALCHEMY_API_KEY not set")?;
    let database = Database::connect(db_url).await?;

    let client = reqwest::Client::new();

    let join = tokio::task::spawn(ChampionTask::new(client, database, alchemy_api_key).run());

    let _ = join.await?;

    // let futures = (0..10)
    //     .map(|x| get_champion(&client, x))
    //     .collect();

    // let res = futures::future::join_all((0..100).map(|i| {
    //     let client = &client;
    //     async move {
    //         let resp = get_champion(client, i).await?;
    //         println!("completed {i}");
    //         Ok::<FighterResponse, reqwest::Error>(resp)
    //     }
    // }))
    // .await;

    // let futs = (0..1000).map(|i| {
    //     let client = &client;
    //     async move {
    //         let resp = get_champion(client, i).await?;
    //         println!("completed {i}");
    //         Ok::<FighterResponse, reqwest::Error>(resp)
    //     }
    // });
    // // let mut futs = vec![];

    // let res = futures::future::join_all(futs).await;

    // for (i, item) in res.into_iter().enumerate() {
    //     let item = item.unwrap();
    //     println!("{} is: {}", i, item.attributes.champion_type);
    // }

    Ok(())

    // for i in 0..100 {
    //     let chamption = get_champion(&client, i).await.unwrap();
    //     println!("{} is: {}", i, chamption.attributes.champion_type);
    // }
}

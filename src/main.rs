
mod client;

use std::time::Duration;
use tokio::time;
use rand::Rng;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    loop {
        if let Err(e) = client::ping().await {
            eprintln!("Ping failed: {}", e);
        }

        let sleep_duration = rand::thread_rng().gen_range(1..=10);
        time::sleep(Duration::from_secs(sleep_duration)).await;
    }
}

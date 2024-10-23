use std::{sync::Arc, time::{Duration, Instant}};

use alloy::{primitives::{Address, U256}, providers::{Provider, ProviderBuilder, WsConnect}};
use rand::Rng;

const WS_URL: &str = "ws://localhost:16014";
const HTTP_URL: &str = "http://localhost:15014";

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let use_ws = std::env::var("USE_WS").is_ok();
    let (sender, receiver) = async_channel::unbounded::<_>();

    for i in 0..(num_cpus::get()) {
        let provider = if use_ws {
            new_ws_provider(WS_URL).await
        } else {
            new_provider(HTTP_URL).await
        };

        tokio::spawn(worker(i, provider, receiver.clone()));
    }

    let mut r = rand::thread_rng();

    let mut interval = tokio::time::interval(Duration::from_millis(10));
    loop {
        interval.tick().await;

        let data = (0..4).into_iter().map(|_| r.gen_range(0..=u64::MAX)).collect::<Vec<_>>();
        let slot = U256::from_limbs(data.try_into().unwrap());
        let address = Address::random();

        sender.send((address, slot)).await.unwrap();
    }

}

async fn worker(
    id: usize,
    provider: Arc<dyn Provider>,
    task: async_channel::Receiver<(Address, U256)>,
) {

    while let Ok((address, slot)) = task.recv().await {
        let start = Instant::now();
        let _ = match provider.get_storage_at(address, slot).await {
            Ok(value) => value,
            Err(e) => {
                eprintln!("[{id}] Got error: {e:?}");
                continue;
            }
        };
        let duration = start.elapsed();

        println!("[{id}] Spent {duration:?} to get value storage at {address:?} {slot:#x}");
    }

    println!("[{id}] Finished");
}

async fn new_provider<T: Into<String>>(url: T) -> Arc<dyn Provider> {
    let p = ProviderBuilder::new()
        .on_http(url.into().parse().expect("Invalid provider url"))
        .boxed();

    Arc::new(p)
}

async fn new_ws_provider<T: Into<String>>(url: T) -> Arc<dyn Provider> {
    let provider = ProviderBuilder::new()
        .on_ws(WsConnect::new(url))
        .await
        .expect("Fail to create ws provider")
        .boxed();

    Arc::new(provider)
}

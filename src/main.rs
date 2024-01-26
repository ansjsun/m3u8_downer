mod worker;

use std::{
    collections::VecDeque,
    fmt::format,
    fs,
    path::Path,
    sync::{Arc, Mutex},
    thread, time,
};

use reqwest::Url;

use crypto::digest::Digest;
use crypto::md5::Md5;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    let url = "https://ddys.pro/getvddr2/video?id=Sx3JyZPjXbE8lm1QeG01oFbsgtyL3js09q3GJeGtKi9beVB9gKOUP4ssxWIvh2UXlifMB6JgX%2B1ubF4%2B50MN7GIzIgEAtBEATsgz285gMZU%3D&type=json";

    worker::Worker::new(url.to_string(), "./hello".to_string())
        .await
        .unwrap()
        .start(10)
        .await;
}

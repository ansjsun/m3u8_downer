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
async fn main() -> Result<()> {
    let url = "https://1252524126.vod2.myqcloud.com/522ff1e0vodcq1252524126/8cedcc78387702299822536361/playlist.f3.m3u8";

    let mut md5 = Md5::new();
    md5.input_str(url);
    let name = md5.result_str();

    down_url(name, url).await
}

async fn down_url(name: String, url: &'static str) -> Result<()> {
    let _ = fs::create_dir(name.as_str());

    let file_name = url.split_inclusive("/").last().unwrap();

    let dir_url: &'static str = &url[..url.len() - file_name.len()];

    let m3u8_list = reqwest::get(url).await?.text().await?;

    let list = Arc::new(Mutex::new(
        m3u8_list
            .split("\n")
            .filter(|v| !v.starts_with("#") && v.trim().len() > 0)
            .map(|s| s.to_string())
            .enumerate()
            .collect::<VecDeque<(usize, String)>>(),
    ));


    tokio::spawn( async {
        loop {
            let num = {
                if let Some(v) = list.lock().unwrap().pop_front() {
                    v
                } else {
                    return;
                }
            };
            goto_down(name.clone(), num.0, format!("{}{}", dir_url, num.1)).await;
        };
    
        let join = thread::spawn(worker);
    
        join.join();
    
        Ok(())
    }
    });

    

async fn goto_down(name: String, id: usize, url: String) -> Result<()> {
    println!("File:{} to downloaded!", id);
    if tokio_dl_stream_to_disk::download(url, Path::new(name.as_str()), format!("{}", id))
        .await
        .is_ok()
    {
        println!("File:{} downloaded successfully!", id);
    }
    Ok(())
}

use std::{collections::VecDeque, sync::Arc};

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Worker {
    base_url: String,
    down_dir: String,
    list: Arc<Mutex<VecDeque<String>>>,
}

impl Worker {
    pub async fn new(url: String, down_dir: String) -> anyhow::Result<Self> {
        let temp_name = url.split_inclusive("/").last().unwrap();

        let base_url = url[..url.len() - temp_name.len()].to_string();

        let m3u8_list = reqwest::get(url).await?.text().await?;

        Ok(Self {
            base_url,
            down_dir,
            list: Arc::new(Mutex::new(
                m3u8_list
                    .split("\n")
                    .filter(|v| !v.starts_with("#") && v.trim().len() > 0)
                    .map(|s| s.to_string())
                    .collect::<VecDeque<String>>(),
            )),
        })
    }

    pub async fn start(&self, thread: u32) {
        let mut join_list = Vec::with_capacity(thread as usize);
        for _ in 0..thread {
            let temp = self.clone();
            join_list.push(tokio::spawn(async move {
                temp.start_job().await;
            }));
        }

        for join in join_list {
            join.await.unwrap();
        }
    }

    pub async fn down(&self, v: &str) -> anyhow::Result<()> {
        let bytes = reqwest::get(format!("{}{}", self.base_url, v))
            .await?
            .bytes()
            .await?;

        Ok(())
    }

    pub async fn start_job(&self) {
        let mut err_num = 0;
        while let Some(v) = self.list.lock().await.pop_front() {
            if err_num > 10 {
                println!("File:{:?} to downloaded err!", v);
                break;
            }
            if let Err(e) = self.down(v.as_str()).await {
                err_num += 1;
                println!("File:{:?} to downloaded err:{:?}!", v, e);
                self.list.lock().await.push_back(v);
                continue;
            }
            err_num = 0;
            println!("File:{:?} to downloaded!", v);
        }
    }
}

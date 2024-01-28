use std::{collections::VecDeque, path::Path, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

#[derive(Clone, Default)]
struct Statics {
    total_num: usize,
    down_num: usize,
    down_size: usize,
    stop: bool,
}

impl Statics {
    pub fn new(total_num: usize) -> Self {
        Self {
            total_num,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Worker {
    base_url: String,
    down_dir: String,
    list: Arc<Mutex<VecDeque<String>>>,
    statics: Arc<Mutex<Statics>>,
}

impl Worker {
    pub async fn new(url: String, down_dir: String) -> anyhow::Result<Self> {
        println!("create worker by url:{:?} down_dir:{:?}", url, down_dir);

        let temp_name = url.split_inclusive("/").last().unwrap();

        let base_url = url[..url.len() - temp_name.len()].to_string();

        let m3u8_list = reqwest::get(url).await?.text().await?;

        std::fs::create_dir_all(down_dir.clone()).unwrap();

        let list = Arc::new(Mutex::new(
            m3u8_list
                .split("\n")
                .filter(|v| !v.starts_with("#") && v.trim().len() > 0)
                .map(|s| s.to_string())
                .collect::<VecDeque<String>>(),
        ));

        let statics = Arc::new(Mutex::new(Statics::new(list.lock().await.len())));

        let temp_statics = statics.clone();
        let temp_down_dir = down_dir.clone();
        tokio::spawn(async move {
            let mut befor_statics = temp_statics.lock().await.clone();
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let statics = temp_statics.lock().await;
                println!(
                    "{:?} processed:{}/{}  Size:{} Speed:{}",
                    temp_down_dir,
                    statics.down_num,
                    statics.total_num,
                    statics.down_size,
                    ((statics.down_size - befor_statics.down_size) as f64) / 2.0 / 1024.0 / 1024.0
                );
                if statics.stop {
                    break;
                }
                befor_statics = statics.clone();
            }
        });

        Ok(Self {
            base_url,
            down_dir,
            list,
            statics,
        })
    }

    pub async fn start(&self, thread: u32) -> &Self {
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

        self
    }

    pub async fn down(&self, v: &str) -> anyhow::Result<()> {
        let file_path = format!("{}/{}", self.down_dir, v);

        if Path::new(&file_path).exists() {
            println!("{} File exists", file_path);
            return Ok(());
        }

        let bytes = tokio::time::timeout(
            Duration::from_secs(30),
            reqwest::get(format!("{}{}", self.base_url, v))
                .await?
                .bytes(),
        )
        .await??;

        //bytes to write file
        let mut file = tokio::fs::File::create(file_path).await?;
        file.write_all(&bytes).await?;

        self.statics.lock().await.down_size += bytes.len();
        Ok(())
    }

    async fn start_job(&self) {
        let mut err_num = 0;

        loop {
            let v = self.list.lock().await.pop_front();
            if v.is_none() {
                break;
            }
            let v = v.unwrap();
            if err_num > 10 {
                println!("File:{:?} to downloaded err!", v);
                break;
            }
            if let Err(_e) = self.down(v.as_str()).await {
                err_num += 1;
                println!("File:{:?} to downloaded err!", v);
                self.list.lock().await.push_back(v);
                continue;
            }
            err_num = 0;

            self.statics.lock().await.down_num += 1;
        }

        self.statics.lock().await.stop = true;
    }

    pub async fn merge_file(&self) -> &Self {
        let mut file_list = std::fs::read_dir(self.down_dir.clone())
            .unwrap()
            .map(|v| v.unwrap().path())
            .collect::<Vec<_>>();
        file_list.sort();

        let path = Path::new(self.down_dir.as_str());

        let path = path.parent().unwrap().join(format!(
            "{}.ts",
            path.file_name().unwrap().to_str().unwrap()
        ));
        let mut file = tokio::fs::File::create(&path).await.unwrap();
        for path in file_list {
            let mut file_temp = tokio::fs::File::open(path).await.unwrap();
            let mut bytes = Vec::new();
            file_temp.read_to_end(&mut bytes).await.unwrap();
            file.write_all(&bytes).await.unwrap();
        }
        println!("File:{:?} to downloaded success!", path);

        std::fs::remove_dir_all(self.down_dir.clone()).unwrap();

        println!("File:{:?} remove dir sucess!", self.down_dir);

        self
    }
}

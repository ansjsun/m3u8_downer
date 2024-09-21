use std::{io::stdout, path::PathBuf};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod worker;

#[tokio::main]
async fn main() {
    // let args: Vec<String> = std::env::args().collect();

    // let url = args.get(1).unwrap().to_string();
    // let name = args.get(2).unwrap().to_string();

    // println!("{:?}", args);

    // worker::Worker::new(url, format!("./{}", name))
    //     .await
    //     .unwrap()
    //     .start(32)
    //     .await
    //     .merge_file()
    //     .await;

    let paths = std::fs::read_dir("D:\\迅雷下载\\video").unwrap();

    for path in paths {
        let mut path = path.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();

        let sub_file = std::fs::read_dir(&path)
            .unwrap()
            .map(|d| d.unwrap().path())
            .collect::<Vec<_>>();

        if sub_file
            .iter()
            .map(|s| s.file_name().unwrap().to_str().unwrap().to_string())
            .find(|n| {
                println!("{:?}", n);
                n == "video"
            })
            .is_some()
        {
            path.push("video");
            // path.push("index");
        } else {
            // path.push("video");
        }

        let index_path = path.join("index.m3u8");
        let list = std::fs::read_to_string(&index_path)
            .unwrap()
            .split("\n")
            .filter(|s| !s.starts_with("#"))
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        merge_dir(&path, name, list).await;
    }
}

async fn merge_dir(path: &PathBuf, name: String, list: Vec<String>) {
    println!("Start merge file:{:?}", path);

    let outpath = path.parent().unwrap().join(format!("\\{}.ts", name));
    let mut file = tokio::fs::File::create(&outpath).await.unwrap();
    for p in list {
        if p.is_empty() {
            continue;
        }
        println!("{:?}", path.join(&p));
        let v = tokio::fs::File::open(path.join(p)).await;
        if v.is_err() {
            continue;
        }
        let mut file_temp = v.unwrap();
        let mut bytes = Vec::new();
        file_temp.read_to_end(&mut bytes).await.unwrap();
        file.write_all(&bytes).await.unwrap();
    }
    println!("File:{:?} to downloaded success!", outpath);
}

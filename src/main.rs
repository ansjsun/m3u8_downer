mod worker;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let url = args.get(1).unwrap().to_string();
    let name = args.get(2).unwrap().to_string();

    println!("{:?}", args);

    worker::Worker::new(url, format!("./{}", name))
        .await
        .unwrap()
        .start(32)
        .await
        .merge_file()
        .await;
}

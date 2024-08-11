use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use tokio::task;
use futures::future::join_all;
use std::process::exit;
use std::{io::stdin, env::consts};
// use ban_rs::conf::read_config;
use ban_rs::helpers;
use ban_rs::{
    watchers::watcher,
    banner::banner,
};


async fn proc(mut rx: mpsc::UnboundedReceiver<String>) {
    println!("start task");
    while let Some(message) =  rx.recv().await {
        sleep(Duration::from_secs(3)).await;
        println!("Received: '{}'", message);
        }
        // println!("second task");
}
        
        // async fn sec_proc() {
            //     println!("first task from secProc");
            // }

#[tokio::main]
async fn main() {
    assert_eq!(consts::OS, "linux", "Only linux is supported");
    assert!(helpers::is_root(), "Please run with root privileges");

    let (main_tx, main_rx) = mpsc::channel(helpers::calculate_optimal_channel_capacity());
    let tasks = vec![watcher(main_tx), banner(record, bantime, main_rx)];

    task::spawn(tasks);


    // let (main_tx, main_rx) = mpsc::unbounded_channel();
    // task::spawn(proc(main_rx), );
    // loop {
    //     let mut input = String::new();
    //     stdin().read_line(&mut input).unwrap();
    //     if input.trim() == "exit" {
    //         break;
    //     } else {
    //         main_tx.send(input.trim().to_string()).unwrap();
    //     }
    // }
}

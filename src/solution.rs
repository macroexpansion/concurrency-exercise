use async_trait::async_trait;
use tokio::{
    sync::{broadcast, mpsc},
    time::{interval, Duration},
};

use crate::statement::*;

const MAX_RETRY: usize = 5;

#[async_trait]
pub trait Solution {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary>;
}

pub struct Solution0;

#[async_trait]
impl Solution for Solution0 {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        let repo_len = repositories.len();

        let (kill_tx, _) = broadcast::channel::<()>(repo_len);
        let (ok_tx, mut ok_rx) = mpsc::channel::<Binary>(1);
        let (err_tx, mut err_rx) = mpsc::channel::<()>(repo_len);
        let mut interval = interval(Duration::from_millis(100));

        let mut tasks = Vec::new();
        for repo in repositories {
            let task = tokio::spawn({
                let ok_tx = ok_tx.clone();
                let err_tx = err_tx.clone();
                let mut kill_rx = kill_tx.subscribe();
                async move {
                    tokio::select! {
                        _ = download_bin(repo.clone(), ok_tx, err_tx) => (),
                        _ = kill_rx.recv() => (),
                    }
                }
            });
            tasks.push(task);
        }

        let mut err_count = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => println!("Another 100ms"),
                Some(bin) = ok_rx.recv() => {
                    kill_tx.send(()).unwrap();
                    for task in tasks {
                        let _ = task.await;
                    }
                    return Some(bin);
                },
                Some(_) = err_rx.recv() => {
                    err_count += 1;
                    if err_count == repo_len {
                        kill_tx.send(()).unwrap();
                        for task in tasks {
                            let _ = task.await;
                        }
                        return None;
                    }
                }
            }
        }
    }
}

async fn download_bin(repo: ServerName, ok_tx: mpsc::Sender<Binary>, err_tx: mpsc::Sender<()>) {
    let mut retry_count = 1;
    // retry MAX_RETRY number of times, else returns error
    while retry_count <= MAX_RETRY {
        let resp = download(repo.clone()).await.ok();

        if resp.is_some() {
            let _ = ok_tx.send(resp.unwrap()).await;
            break;
        } else {
            println!("{:?} returns error, retry downloading...", repo);
            retry_count += 1;
        }
    }

    println!("{:?} max number of retries reached", repo);
    let _ = err_tx.send(()).await;
}

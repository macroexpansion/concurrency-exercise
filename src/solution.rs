use async_trait::async_trait;
use tokio::{
    sync::mpsc,
    time::{interval, Duration},
};

use crate::statement::*;

#[async_trait]
pub trait Solution {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary>;
}

pub struct Solution0;

#[async_trait]
impl Solution for Solution0 {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        let repo_len = repositories.len();

        let (ok_tx, mut ok_rx) = mpsc::channel::<Binary>(1);
        let (err_tx, mut err_rx) = mpsc::channel::<()>(repositories.len());
        let mut interval = interval(Duration::from_millis(100));

        for repo in repositories {
            let ok_tx = ok_tx.clone();
            let err_tx = err_tx.clone();
            let _ = tokio::spawn(async move {
                let resp = download(repo).await.ok();

                if resp.is_some() {
                    let _ = ok_tx.send(resp.unwrap()).await;
                } else {
                    let _ = err_tx.send(()).await;
                }
            });
        }

        let mut err_count = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => println!("Another 100ms"),
                Some(bin) = ok_rx.recv() => {
                    return Some(bin);
                },
                Some(_) = err_rx.recv() => {
                    err_count += 1;
                    if err_count == repo_len {
                        return None;
                    }
                }
            }
        }
    }
}

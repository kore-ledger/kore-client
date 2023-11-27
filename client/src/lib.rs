mod database;
mod http;
pub mod settings;
mod kore;

use ::futures::Future;
use database::leveldb::{LDBCollection, LevelDBManager};
use settings::ClientSettings;

use std::error::Error;

use kore_base::{Node, Notification};
use tokio_util::sync::CancellationToken;

use database::leveldb;

pub struct Client {
    kore_node: Node<LevelDBManager, LDBCollection>,
    cancellation_token: CancellationToken,
}

impl Client {
    pub fn build(settings: ClientSettings) -> Result<Self, Box<dyn Error>> {
        let cancellation_token = CancellationToken::new();

        let (kore_node, kore_api, keys) = kore::build(&settings, cancellation_token.clone())?;

        if settings.http {
            http::build(settings, kore_api, keys, cancellation_token.clone());
        }

        Ok(Client {
            kore_node,
            cancellation_token,
        })
    }

    pub fn bind_with_shutdown(&self, shutdown_signal: impl Future + Send + 'static) {
        let cancellation_token = self.cancellation_token.clone();
        tokio::spawn(async move {
            shutdown_signal.await;
            log::info!("Shutdown signal received");
            cancellation_token.cancel();
        });
    }

    pub async fn run<H>(self, notifications_handler: H)
    where
        H: Fn(Notification),
    {
        self.kore_node
            .handle_notifications(notifications_handler)
            .await;
        self.cancellation_token.cancel();
        log::info!("Stopped");
    }
}

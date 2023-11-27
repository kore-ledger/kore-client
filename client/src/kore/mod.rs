use std::path::Path;

use kore_base::{crypto::KeyPair, Api, Node};
use tokio_util::sync::CancellationToken;

use crate::{
    leveldb::{open_db, LDBCollection, LevelDBManager},
    ClientSettings,
};

pub fn build(
    settings: &ClientSettings,
    cancellation_token: CancellationToken,
) -> Result<(Node<LevelDBManager, LDBCollection>, Api, KeyPair), kore_base::Error> {
    let db = {
        let db = open_db(Path::new(&settings.db_path));
        LevelDBManager::new(db)
    };

    let keys = {
        let derivator = &settings.taple.node.key_derivator;
        let secret_key = &settings.taple.node.secret_key;
        KeyPair::from_hex(derivator, secret_key).expect("Key derivated")
    };

    let (taple_node, taple_api) = Node::build(settings.taple.clone(), db)?;

    taple_node.bind_with_shutdown(async move {
        cancellation_token.cancelled().await;
    });

    Ok((taple_node, taple_api, keys))
}

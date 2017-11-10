extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_DEL_ASSETS_ID};
use service::wallet::Asset;
use service::schema::wallet::WalletSchema;

pub const FEE_FOR_MINING: u64 = 1;

message! {
    struct TxDelAsset {
        const TYPE = SERVICE_ID;
        const ID = TX_DEL_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey  [00 => 32]
        field assets:      Vec<Asset>  [32 => 40]
        field seed:        u64         [40 => 48]
    }
}

impl Transaction for TxDelAsset {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        let creator = schema.wallet(self.pub_key());
        if let Some(mut creator) = creator {

            if creator.balance() >= FEE_FOR_MINING {
                creator.decrease(FEE_FOR_MINING);
                println!("Asset {:?}", self.assets());
                creator.del_assets(self.assets());
                println!("Wallet after delete assets: {:?}", creator);
                schema.wallets().put(self.pub_key(), creator)
            }
        }

    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": FEE_FOR_MINING,
        })
    }
}


#[cfg(test)]
use service::wallet::Wallet;
#[cfg(test)]
use exonum::storage::{MemoryDB, Database};

#[cfg(test)]
fn get_json() -> String {
    r#"{
  "body": {
    "pub_key": "1d9c731ebac3d7da9482470ae8b13a839cb05ef4f21f8d119e2c4bf175333cf7",
    "assets": [
      {
        "hash_id": "asset_1",
        "amount": 45
      },
      {
        "hash_id": "asset_2",
        "amount": 17
      }
    ],
    "seed": "113"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 4,
  "signature": "e7a3d71fc093f9ddaba083ba3e1618514c96003d9a01cdf6d5c0da344f12c800db9e7b210f9a7b372ddd7e57f299d8bc0e55d238ad1fa6b9d06897c2bda29901"
}"#.to_string()
}

#[test]
fn test_convert_from_json() {
    let tx_del: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();
    assert!(tx_del.verify());
    assert_eq!(45, tx_del.assets()[0].amount());
    assert_eq!("asset_2", tx_del.assets()[1].hash_id());
}

#[test]
fn positive_delete_assets_test() {
    let tx_del: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let assets = vec![
        Asset::new("asset_1", 100),
        Asset::new("asset_2", 17),
    ];

    let wallet = Wallet::new(tx_del.pub_key(), 100, assets);
    wallet_schema.wallets().put(tx_del.pub_key(), wallet);

    tx_del.execute(&mut wallet_schema.view);

    if let Some(wallet) = wallet_schema.wallet(tx_del.pub_key()) {
        assert!(wallet.in_wallet_assets(vec![
            Asset::new("asset_1", 55)
        ]));
        assert!(!wallet.in_wallet_assets(vec![
            Asset::new("asset_2", 0)
        ]));
    } else {
        panic!("Something wrong!!!");
    }
}

#[test]
fn negative_delete_assets_test() {
    let tx_del: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let assets = vec![
        Asset::new("asset_1", 400),
    ];

    let wallet = Wallet::new(tx_del.pub_key(), 100, assets);
    wallet_schema.wallets().put(tx_del.pub_key(), wallet);

    tx_del.execute(&mut wallet_schema.view);

    if let Some(wallet) = wallet_schema.wallet(tx_del.pub_key()) {
        assert!(wallet.in_wallet_assets(vec![
            Asset::new("asset_1", 400)
        ]));
    } else {
        panic!("Something wrong!!!");
    }
}

#[test]
fn add_asset_info_test() {
    let tx: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();
    assert_eq!(FEE_FOR_MINING, tx.info()["tx_fee"]);
}

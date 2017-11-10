extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use service::wallet::Asset;
use serde_json::Value;

use super::{SERVICE_ID, TX_TRANSFER_ID};
use super::schema::wallet::WalletSchema;
use super::schema::transaction_status::{TxStatusSchema, TxStatus};

pub const FEE_FOR_TRANSFER: u64 = 1;

message! {
    struct TxTransfer {
        const TYPE = SERVICE_ID;
        const ID = TX_TRANSFER_ID;
        const SIZE = 88;

        field from:        &PublicKey  [00 => 32]
        field to:          &PublicKey  [32 => 64]
        field amount:      u64         [64 => 72]
        field assets:      Vec<Asset>  [72 => 80]
        field seed:        u64         [80 => 88]
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        let mut tx_status = TxStatus::Fail;
        if let Some(mut sender) = schema.wallet(self.from()) {
            let amount = self.amount();
            let update_amount = amount == 0 && sender.balance() >= FEE_FOR_TRANSFER ||
                amount > 0 && sender.balance() >= amount + FEE_FOR_TRANSFER;
            let update_assets = self.assets().is_empty() ||
                !self.assets().is_empty() && sender.in_wallet_assets(self.assets());
            if update_amount && update_assets {
                sender.decrease(amount + FEE_FOR_TRANSFER);
                sender.del_assets(self.assets());
                let mut receiver = schema.create_wallet(self.to());
                receiver.increase(amount);
                receiver.add_assets(self.assets());

                println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
                let mut wallets = schema.wallets();
                wallets.put(self.from(), sender);
                wallets.put(self.to(), receiver);
                tx_status = TxStatus::Success;
            }
        }
        let mut tx_status_schema = TxStatusSchema { view: schema.view };
        tx_status_schema.set_status(&self.hash(), tx_status);
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": FEE_FOR_TRANSFER,
        })
    }
}

#[cfg(test)]
use exonum::storage::{MemoryDB, Database};
#[cfg(test)]
use service::wallet::Wallet;


#[cfg(test)]
fn get_json() -> String {
    r#"{
  "body": {
    "from": "739fe1c8507aac54b5d4af116544fec304cf8b0f759d0bce39a7934630c0457e",
    "to": "c08575875170900ac946fc9c0c521bea3d61c138380512cc8d1f55ba27289d27",
    "amount": "3",
    "assets": [
      {
        "hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",
        "amount": 3
      }
    ],
    "seed": "123"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 2,
  "signature": "4f9c0a9ddb32a1d8e61d3b656dec5786fb447c19362853ddac67a2c4f48c9ad65a377ee86a02727a27a35d16a14dea84f6920878ab82a6e850e8e7814bb64701"
}"#.to_string()
}

#[test]
fn test_convert_from_json() {
    let tx: TxTransfer = ::serde_json::from_str(&get_json()).unwrap();
    assert!(tx.verify());
    assert_eq!(
        Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",3),
        tx.assets()[0]
    );
    assert_eq!(3, tx.amount());
}

#[test]
fn positive_send_staff_test() {
    let tx: TxTransfer = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let from = Wallet::new(
        tx.from(),
        100,
        vec![
            Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",100),
        ],
    );
    wallet_schema.wallets().put(tx.from(), from);

    tx.execute(&mut wallet_schema.view);

    let from = wallet_schema.wallet(tx.from());
    let to = wallet_schema.wallet(tx.to());
    if let (Some(from), Some(to)) = (from, to) {
        assert_eq!(96, from.balance());
        assert_eq!(3, to.balance());
        assert_eq!(
            vec![Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", 97),],
            from.assets()
        );
        assert_eq!(
            vec![
                Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", 3),
            ],
            to.assets()
        );
    } else {
        panic!("Something wrong!!!");
    }

}
#[test]
fn transfer_info_test() {
    let tx: TxTransfer = ::serde_json::from_str(&get_json()).unwrap();
    assert_eq!(FEE_FOR_TRANSFER, tx.info()["tx_fee"]);
}

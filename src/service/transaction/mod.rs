use super::schema;
use super::wallet;

use super::SERVICE_ID;
//const SERVICE_ID: u16 = 1;

// Create Wallet
pub const TX_CREATE_WALLET_ID: u16 = 1;
const INIT_BALANCE: u64 = 100;
pub mod create_wallet;

// Transfer
pub const TX_TRANSFER_ID: u16 = 2;
pub mod transfer;

// Add Assets
pub const TX_ADD_ASSETS_ID: u16 = 3;
pub mod add_assets;

// Add Assets
pub const TX_DEL_ASSETS_ID: u16 = 4;
pub mod del_assets;

// Buy Transaction
pub const TX_TRADE_ASSETS_ID: u16 = 5;
pub mod trade_assets;

// Buy Transaction
pub const TX_EXCHANGE_ID: u16 = 6;
pub mod exchange;

// Mining coin
pub const TX_MINING_ID: u16 = 7;
const AMOUNT_MINING_COIN: u64 = 100_000_000_000_000;
pub mod mining;

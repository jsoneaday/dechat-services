use futures::stream::StreamExt;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use sui_sdk::rpc_types::EventFilter;
use sui_sdk::{SuiClient, SuiClientBuilder};
use move_core_types::language_storage::{StructTag, TypeTag};

#[allow(unused)]
const DEVNET_URL: &str = "https://fullnode.devnet.sui.io:443";
const TESTNET_URL: &str = "https://fullnode.testnet.sui.io:443";
const TESTNET_WS_URL: &str = "wss://rpc.testnet.sui.io:443";

#[allow(unused)]
pub struct SuiEventHandler {
    sui_client: SuiClient
}

#[allow(unused)]
impl SuiEventHandler {
    async fn init() -> Self {
        SuiEventHandler { 
            sui_client: SuiClientBuilder::default()
                .ws_url(TESTNET_WS_URL)
                .build(TESTNET_URL)
                .await
                .unwrap() 
        }
    }

    async fn subscribe_event(&self, address: AccountAddress, module: Identifier, name: Identifier, type_params: Vec<TypeTag>) {       
        let mut subscribe = self.sui_client.event_api().subscribe_event(EventFilter::MoveEventType(StructTag { address, module, name, type_params })).await.unwrap();
        loop {
            println!("{:?}", subscribe.next().await);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sui::wallet::retrieve_wallet;
    use super::*;

    #[tokio::test]
    async fn test_subscribe_event() {
        let client = SuiEventHandler::init().await;
        let module = Identifier::new("dechat_sui").unwrap();
        let name = Identifier::new("CreateProfileEvent").unwrap();
        let type_params = vec![];
        let mut wallet = retrieve_wallet().await.unwrap();
        let active_address = wallet.active_address().unwrap();

        client.subscribe_event(active_address.into(), module, name, type_params).await;
    }
}
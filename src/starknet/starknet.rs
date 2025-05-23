use std::str::FromStr;

use bevy::prelude::*;
use starknet::{
    accounts::{Account, ConnectedAccount, SingleOwnerAccount},
    core::{
        types::{BlockId, BlockTag, Call, ExecutionResult, Felt},
        utils::get_selector_from_name,
    },
    providers::{JsonRpcClient, Provider, Url, jsonrpc::HttpTransport},
    signers::{LocalWallet, SigningKey},
};
use std::sync::Arc;
use tokio::{
    sync::mpsc,
    time::{Duration, sleep},
};

use bevy_enhanced_input::prelude::*;

use crate::systems::input::StartGame;

use super::config::{
    GAME_MINT_CONTRACT_ADDRESS, GAME_SYSTEMS_CONTRACT_ADDRESS, PLAYER_ONE_ADDRESS,
    PLAYER_ONE_PRIVATE_KEY, RPC_URL,
};
use super::tokio::{TokioRuntimeResource, TokioRuntimeState};

#[derive(Resource)]
pub struct StarknetChannel {
    tx: mpsc::Sender<StarknetCommands>,
}

pub enum StarknetCommands {
    SendStartGameTx,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum StarknetServerState {
    #[default]
    NotReady,
    Ready,
}

pub struct StarknetPlugin;
impl Plugin for StarknetPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<StarknetServerState>();
        app.add_systems(
            OnEnter(TokioRuntimeState::Ready),
            spawn_starknet_caller_thread,
        );
        app.add_observer(handle_start_game_action);
    }
}

fn handle_start_game_action(trigger: Trigger<Started<StartGame>>, channel: Res<StarknetChannel>) {
    if trigger.value {
        info!("StartGame action triggered - sending Starknet transaction");
        let _ = channel.tx.try_send(StarknetCommands::SendStartGameTx);
    }
}

fn spawn_starknet_caller_thread(
    mut commands: Commands,
    rt: Res<TokioRuntimeResource>,
    mut next_state: ResMut<NextState<StarknetServerState>>,
) {
    let (tx, mut rx) = mpsc::channel::<StarknetCommands>(64);

    let _ = rt.0.spawn(async move {
        let provider = get_rpc_provider().await;
        let (signer, address) = get_player_account();
        let chain_id = provider.chain_id().await.unwrap();

        let processing_lock = Arc::new(tokio::sync::Mutex::new(()));

        info!("Started STARKNET TX SENDING SERVER...");

        while let Some(starknet_command) = rx.recv().await {
            // Try to acquire the lock - if already locked, skip this command
            if let Ok(lock) = processing_lock.try_lock() {
                info!("{:?}", lock);
                match starknet_command {
                    StarknetCommands::SendStartGameTx => {
                        let player_account = create_player_account(
                            provider.clone(),
                            signer.clone(),
                            address,
                            chain_id,
                        );

                        match mint_token(&player_account).await {
                            Ok(adventurer_id_hex) => {
                                if let Ok(adventurer_id) = Felt::from_hex(&adventurer_id_hex) {
                                    // Wait for the mint transaction to be processed
                                    sleep(Duration::from_secs(15)).await;

                                    // Use the same account instance for the second transaction
                                    // The account should manage the nonce internally
                                    send_start_game_tx(&player_account, adventurer_id).await;
                                } else {
                                    error!(
                                        "Failed to parse adventurer ID hex: {}",
                                        adventurer_id_hex
                                    );
                                }
                            }
                            Err(e) => error!("Mint token failed: {}", e),
                        }
                    }
                }
            } else {
                info!("Already processing a transaction, skipping new request");
            }
        }
    });

    commands.insert_resource(StarknetChannel { tx });
    next_state.set(StarknetServerState::Ready);
}

async fn get_rpc_provider() -> JsonRpcClient<HttpTransport> {
    JsonRpcClient::new(HttpTransport::new(Url::parse(RPC_URL).unwrap()))
}

fn get_player_account() -> (LocalWallet, Felt) {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex(PLAYER_ONE_PRIVATE_KEY).unwrap(),
    ));
    let address = Felt::from_hex(PLAYER_ONE_ADDRESS).unwrap();
    (signer, address)
}

fn create_player_account(
    provider: JsonRpcClient<HttpTransport>,
    signer: LocalWallet,
    address: Felt,
    chain_id: Felt,
) -> SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> {
    let mut account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id,
        starknet::accounts::ExecutionEncoding::New,
    );
    account.set_block_id(BlockId::Tag(BlockTag::Latest));
    account
}

async fn wait_for_tx_acceptance(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: Felt,
) -> Result<(), String> {
    let mut retries = 60;
    let delay = Duration::from_secs(4);

    loop {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(receipt) => match receipt.receipt.execution_result() {
                ExecutionResult::Succeeded => {
                    info!("Transaction {:?} accepted", tx_hash);
                    return Ok(());
                }
                ExecutionResult::Reverted { reason } => {
                    return Err(format!("Transaction reverted: {}", reason));
                }
            },
            Err(e) => {
                if retries == 0 {
                    return Err(format!("Failed to confirm tx after retries: {}", e));
                }
                retries -= 1;
                sleep(delay).await;
            }
        }
    }
}

async fn mint_token(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<String, String> {
    let provider = account.provider();

    match account.get_nonce().await {
        Ok(nonce) => info!("Nonce before mint: {:?}", nonce),
        Err(e) => error!("Failed to get nonce before mint: {:?}", e),
    }

    let tx = account
        .execute_v3(vec![Call {
            to: Felt::from_hex(GAME_MINT_CONTRACT_ADDRESS).unwrap(),
            selector: get_selector_from_name("mint").unwrap(),
            calldata: vec![
                Felt::from_hex_unchecked("341104419177"),
                Felt::from_hex_unchecked("0"),
                Felt::from_hex_unchecked("1"),
                Felt::from_hex_unchecked("1"),
                Felt::from_hex_unchecked(PLAYER_ONE_ADDRESS),
            ],
        }])
        .send()
        .await
        .map_err(|e| format!("Minting transaction failed: {}", e))?;

    info!("Mint transaction sent with hash: {:?}", tx.transaction_hash);

    wait_for_tx_acceptance(provider, tx.transaction_hash).await?;

    match provider.get_transaction_receipt(tx.transaction_hash).await {
        Ok(receipt) => {
            info!(
                "Transaction receipt received: {:?}",
                receipt.receipt.transaction_hash()
            );
            for (i, event) in receipt.receipt.events().iter().enumerate() {
                info!("Event {}: {:?}", i, event.data);
                if let Some(item) = event.data.get(3) {
                    let id_hex_str = format!("{:#x}", item);
                    info!("Extracted adventurer ID: {}", id_hex_str);

                    match account.get_nonce().await {
                        Ok(nonce) => info!("Nonce after mint: {:?}", nonce),
                        Err(e) => error!("Failed to get nonce after mint: {:?}", e),
                    }

                    return Ok(id_hex_str);
                }
            }
            Err("Adventurer ID not found in mint event".to_string())
        }
        Err(err) => Err(format!("Failed to get transaction receipt: {}", err)),
    }
}

async fn send_start_game_tx(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    adventurer_id: Felt,
) {
    let adventurer_id_str = format!("{}", adventurer_id.to_string());
    info!(
        "Sending start game tx with adventurer ID: {}",
        adventurer_id_str
    );

    // Get and log current nonce (for debugging)
    match account.get_nonce().await {
        Ok(nonce) => info!("Nonce before start_game: {:?}", nonce),
        Err(e) => error!("Failed to get nonce before start_game: {:?}", e),
    }

    let tx_exec = account
        .execute_v3(vec![Call {
            to: Felt::from_hex(GAME_SYSTEMS_CONTRACT_ADDRESS).unwrap(),
            selector: get_selector_from_name("start_game").unwrap(),
            calldata: vec![adventurer_id, Felt::from_str("12").unwrap()],
        }])
        .send()
        .await;

    let provider = account.provider();
    match &tx_exec {
        Ok(result) => {
            info!(
                "Start game tx sent successfully with hash: {:?}",
                result.transaction_hash
            );
            if let Err(e) = wait_for_tx_acceptance(provider, result.transaction_hash).await {
                error!("Start game tx not confirmed: {}", e);
            }
        }
        Err(e) => error!("Start game tx failed: {:?}", e),
    }
}

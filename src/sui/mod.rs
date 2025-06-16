mod helpers;
use crate::sui::helpers::setup_for_write;
use anyhow::anyhow;
use serde_json::json;
use shared_crypto::intent::Intent;
use std::error::Error;
use sui_config::{SUI_KEYSTORE_FILENAME, sui_config_dir};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::json::SuiJsonValue;
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_sdk::types::Identifier;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_sdk::types::transaction::{Argument, CallArg, Command, Transaction, TransactionData};
use tracing::info;

// hold image data through WALRUS
pub async fn start_client() -> Result<(), Box<dyn Error>> {
    let (sui, sender, _) = setup_for_write().await?;

    let coins = sui
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await?;
    let coin = coins.data.into_iter().next().unwrap();

    let mut ptb = ProgrammableTransactionBuilder::new();

    let nft_name = "My NFT Name";
    let input_argument = CallArg::Pure(bcs::to_bytes(&nft_name).unwrap());

    // Add this input to the builder
    ptb.input(input_argument)?;

    let nft_desc = "description";
    let input_argument = CallArg::Pure(bcs::to_bytes(&nft_desc).unwrap());

    // Add this input to the builder
    ptb.input(input_argument)?;

    let nft_url = "url";
    let input_argument = CallArg::Pure(bcs::to_bytes(&nft_url).unwrap());

    // Add this input to the builder
    ptb.input(input_argument)?;

    let pkg_id = "0x87e1d6f71d7caa286ebab6dcb217d9426777112c2426fe8ef1ca3abacd78b179";
    let package = ObjectID::from_hex_literal(pkg_id).map_err(|e| anyhow!(e))?;
    let module = Identifier::new("nft").map_err(|e| anyhow!(e))?;
    let function = Identifier::new("mint_to_sender").map_err(|e| anyhow!(e))?;
    ptb.command(Command::move_call(
        package,
        module,
        function,
        vec![],
        vec![Argument::Input(0), Argument::Input(1), Argument::Input(2)],
    ));
    let builder = ptb.finish();

    let gas_budget = 10_000_000;
    let gas_price = sui.read_api().get_reference_gas_price().await?;
    // create the transaction data that will be sent to the network
    let tx_data = TransactionData::new_programmable(
        sender,
        vec![coin.object_ref()],
        builder,
        gas_budget,
        gas_price,
    );
    let keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(&sender, &tx_data, Intent::sui_transaction())?;
    info!("signature {:?}", signature);
    info!("Executing the transaction...");
    let transaction_response = sui
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![signature]),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;
    info!("{:?}", transaction_response);
    Ok(())
}

use sp_keyring::AccountKeyring;
use subxt::{
    ClientBuilder,
    DefaultConfig,
    PairSigner,
    PolkadotExtrinsicParams,
};

#[subxt::subxt(runtime_metadata_path = "polkadot_metadata.scale")]
pub mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let api = ClientBuilder::new()
        .set_url("ws://127.0.0.1:9944")
        .build()
        .await?
        .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();

    tokio::spawn(async move {
        let dest = AccountKeyring::Bob.to_account_id().into();

        // Obtain an extrinsic, calling the "transfer" function in
        // the "balances" pallet.
        let extrinsic = match api.tx().balances().transfer(dest, 123_456_789_012_345) {
            Ok(extrinsic) => extrinsic,
            Err(_) => { return }
        };

        // Sign and submit the extrinsic, returning its hash.
        let tx_hash = match extrinsic.sign_and_submit_default(&signer).await {
            Ok(tx_hash) => tx_hash,
            Err(_) => { return }
        };

        println!("Balance transfer extrinsic submitted: {}", tx_hash);
    }).await?;

    Ok(())
}

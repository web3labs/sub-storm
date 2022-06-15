#![allow(clippy::needless_range_loop)]

use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, DefaultConfig, PairSigner, PolkadotExtrinsicParams};
use clap::Parser;

const TX_POOL_LIMIT: usize = 8192;

#[subxt::subxt(runtime_metadata_path = "polkadot_metadata.scale")]
pub mod polkadot {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    ws_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let api = ClientBuilder::new()
        .set_url(args.ws_url)
        .build()
        .await?
        .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();

    let alice = AccountKeyring::Alice.to_account_id();
    let alice_acc = api.storage().system().account(&alice, None).await?;

    let mut signer = PairSigner::new(AccountKeyring::Alice.pair());
    let dest = AccountKeyring::Bob.to_account_id();

    signer.set_nonce(alice_acc.nonce);

    let num_steps = TX_POOL_LIMIT - 2;
    let mut vec = Vec::with_capacity(num_steps);

    for index in 0..num_steps {
        let extrinsic = api
            .tx()
            .balances()
            .transfer(dest.clone().into(), 123_456_789_012_445)?;

        let encoded = extrinsic
            .create_signed(&signer, Default::default())
            .await
            .unwrap();

        vec.push(encoded);

        signer.increment_nonce();

        println!("Step {}", index);
    }

    for index in 0..num_steps {
        if let Err(e) = api.client.rpc().submit_extrinsic(vec[index].clone()).await {
            println!("Step {index} failed: {e}");
        }

        println!("Step {}", index);
    }

    Ok(())
}

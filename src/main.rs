#![allow(clippy::needless_range_loop)]

use std::{thread::sleep, time::Duration};

use clap::Parser;
use polkadot::RuntimeApi;
use sp_keyring::{sr25519::sr25519::Pair, AccountKeyring};
use subxt::{ClientBuilder, DefaultConfig, PairSigner, PolkadotExtrinsicParams};

const TX_POOL_LIMIT: usize = 8192;
const TX_BATCH_SIZE: usize = TX_POOL_LIMIT;
const SLEEP_BETWEEN_BATCHES_SECONDS: u64 = 6;

#[subxt::subxt(runtime_metadata_path = "polkadot_metadata.scale")]
pub mod polkadot {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    ws_url: String,
}

async fn reset_alice_nonce(
    api: &RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>,
    signer: &mut PairSigner<DefaultConfig, Pair>,
) -> Result<(), Box<dyn std::error::Error>> {
    let alice = AccountKeyring::Alice.to_account_id();

    let alice_acc = api.storage().system().account(&alice, None).await?;
    signer.set_nonce(alice_acc.nonce);

    Ok(())
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

    let mut signer = PairSigner::new(AccountKeyring::Alice.pair());
    let dest = AccountKeyring::Bob.to_account_id();

    reset_alice_nonce(&api, &mut signer).await?;

    let mut current_batch_size = 0;
    for index in 1..u64::MAX {
        let extrinsic = api
            .tx()
            .balances()
            .transfer(dest.clone().into(), 1)?;

        println!("Sent tx number {}", index);

        match extrinsic.sign_and_submit_default(&signer).await {
            Ok(tx_hash) => {
                println!("Got Result Ok({tx_hash:})");

                signer.increment_nonce();

                current_batch_size += 1;
            }
            Err(message) => {
                println!("Got Result Err({message:})");
                println!("Will sleep for {} seconds", SLEEP_BETWEEN_BATCHES_SECONDS);

                sleep(Duration::from_secs(SLEEP_BETWEEN_BATCHES_SECONDS));

                /*
                    For some reason the extrinsics flow breaks and the local nonce keeps increasing,
                    whereas the on-chain nonce stays the same. That indicates that an extrinsic may
                    be lost. In order to counter that, we'll manually re-fetch the nonce, so we can
                    be sure that the flow is contiguous.
                */
                reset_alice_nonce(&api, &mut signer).await?;

                current_batch_size = 0;
            }
        }

        if current_batch_size != 0 && current_batch_size % TX_BATCH_SIZE as u64 == 0 {
            println!("Will sleep for {} seconds", SLEEP_BETWEEN_BATCHES_SECONDS);

            sleep(Duration::from_secs(SLEEP_BETWEEN_BATCHES_SECONDS));

            current_batch_size = 0;
        }
    }

    Ok(())
}

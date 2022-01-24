// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with subxt.  If not, see <http://www.gnu.org/licenses/>.

//! To run this example, a local polkadot node should be running.
//!
//! E.g.
//! ```bash
//! curl "https://github.com/paritytech/polkadot/releases/download/v0.9.11/polkadot" --output /usr/local/bin/polkadot --location
//! polkadot --dev --tmp
//! ```

use beefy_light_client::commitment;
use subxt::{BeefySubscription, ClientBuilder};

#[subxt::subxt(runtime_metadata_path = "examples/polkadot_metadata.scale")]
pub mod polkadot {}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let api = ClientBuilder::new()
        .build()
        .await?
        .to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>();

    let sub = api.client.rpc().subscribe_beefy_justifications().await?;
    let mut sub = BeefySubscription::new(sub);

    let raw = sub.next().await.unwrap();
    let data =
        <commitment::SignedCommitment as codec::Decode>::decode(&mut &raw.0[..]).unwrap();

    let commitment::Commitment {
        payload,
        block_number,
        validator_set_id,
    } = data.commitment;
    println!("signed commitment block_number : {}", block_number);
    println!("signed commitment validator_set_id : {}", validator_set_id);
    println!("signed commitment payload : {:?}", payload);

    for signature in data.signatures.iter() {
        println!("signature :  {:?}", signature);
    }
    Ok(())
}

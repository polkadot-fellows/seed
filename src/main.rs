#[subxt::subxt(runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443")]
pub mod kusama {
    #[subxt(substitute_type = "sp_runtime::multiaddress::MultiAddress")]
    use ::subxt::ext::sp_runtime::MultiAddress;
}

use anyhow::Result;
use kusama::runtime_types::{
    kusama_runtime::RuntimeCall, pallet_collective::pallet::Call as TechnicalCommitteeCall,
    pallet_ranked_collective::pallet::Call as CollectiveCall,
    pallet_utility::pallet::Call as UtilityCall,
};
use parity_scale_codec::Encode as _;
use std::io::BufRead;
use std::{fs, io, str::FromStr as _};
use subxt::ext::sp_core;
use subxt::ext::sp_runtime::{AccountId32, MultiAddress};

fn main() -> Result<()> {
    let members = load_members()?;
    assert_eq!(45, members.len());

    let mut calls = Vec::new();

    for member in members.iter() {
        if member.name == "gavofyork" {
            // gav has already been added
            continue;
        }
        calls.push(RuntimeCall::FellowshipCollective(
            CollectiveCall::add_member {
                who: MultiAddress::Id(member.account_id.clone()),
            },
        ));
        for rank in 0..member.rank {
            // referendum required to promote to rank VII and higher
            if rank < 6 {
                calls.push(RuntimeCall::FellowshipCollective(
                    CollectiveCall::promote_member {
                        who: MultiAddress::Id(member.account_id.clone()),
                    },
                ))
            }
        }
    }

    let proposal = RuntimeCall::Utility(UtilityCall::batch { calls });
    let proposal_hash = sp_core::blake2_256(&proposal.encode());
    let length_bound = proposal.encoded_size() as u32;
    let call = RuntimeCall::TechnicalCommittee(TechnicalCommitteeCall::propose {
        proposal: Box::new(proposal),
        threshold: 2,
        length_bound,
    });

    let bytes = call.encode();

    println!("Call data: 0x{}", hex::encode(bytes));
    println!("Proposal hash: 0x{}", hex::encode(proposal_hash));

    Ok(())
}

#[derive(Debug)]
struct Member {
    name: String,
    account_id: AccountId32,
    rank: u8,
}

fn load_members() -> Result<Vec<Member>> {
    let mut members = Vec::new();

    let regex = regex::Regex::new(r#"\| \[(.*)\].*\| `(.*)` \| (\d) \|"#)?;
    let readme = fs::File::open("./README.md")?;
    for line in io::BufReader::new(readme).lines() {
        if let Some(captures) = regex.captures(&line?) {
            let name = captures[1].to_owned();
            let account_id = &captures[2];
            let account_id = AccountId32::from_str(account_id)
                .map_err(|e| anyhow::anyhow!("Error parsing account id {}: {}", account_id, e))?;
            let rank = u8::from_str(&captures[3])?;
            members.push(Member {
                name,
                account_id,
                rank,
            })
        }
    }
    Ok(members)
}

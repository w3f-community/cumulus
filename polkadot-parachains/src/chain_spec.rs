// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use rococo_parachain_runtime::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use polkadot_parachain_primitives::{DOT, BTC, ETH, KONO};
use sp_runtime::FixedU128;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<rococo_parachain_runtime::GenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the shell parachain runtime.
pub type ShellChainSpec = sc_service::GenericChainSpec<shell_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_chain_spec(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				hex!["e2b2d3e7c3931a4562feaa27c22e858dea0bf2828bbab28c0b799f61eb0b9462"].into(),
				vec![
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
				],
				vec![
					hex!["e2b2d3e7c3931a4562feaa27c22e858dea0bf2828bbab28c0b799f61eb0b9462"].into(),
					// These are for the oracle
					// ETH
					hex!["2463e932d63a263395ac6730abd3a22049100f25a7cf7a3bcce8d5c9e9875a33"].into(),
					// DOT
					hex!["16ae36476c937cfb1f970e3d374e23bfbff23b67c3224cf9b934105378dc1278"].into(),
					// KONO
					hex!["8ad57abb27cf9d6b067e8f8ec856600a71e20647884a52e10ad9e5af3f00013a"].into(),
					// BTC
					hex!["64f0bdf9ca65acf36df6189aab420ee2d6b07ac05f90a01744ede75c88a36721"].into(),
					get_account_id_from_seed::<sr25519::Public>("Alice"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: id.into(),
		},
	)
}

pub fn get_shell_chain_spec(id: ParaId) -> ShellChainSpec {
	ShellChainSpec::from_genesis(
		"Shell Local Testnet",
		"shell_local_testnet",
		ChainType::Local,
		move || shell_testnet_genesis(id),
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn staging_test_net(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
				vec![
					// $secret//one
					hex!["aad9fa2249f87a210a0f93400b7f90e47b810c6d65caa0ca3f5af982904c2a33"]
						.unchecked_into(),
					// $secret//two
					hex!["d47753f0cca9dd8da00c70e82ec4fc5501a69c49a5952a643d18802837c88212"]
						.unchecked_into(),
				],
				vec![
					hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
				],
				id,
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> rococo_parachain_runtime::GenesisConfig {
	let btc_oracle_account: AccountId = hex!["64f0bdf9ca65acf36df6189aab420ee2d6b07ac05f90a01744ede75c88a36721"].into();
	let eth_oracle_account: AccountId = hex!["2463e932d63a263395ac6730abd3a22049100f25a7cf7a3bcce8d5c9e9875a33"].into();
	let dot_oracle_account: AccountId = hex!["16ae36476c937cfb1f970e3d374e23bfbff23b67c3224cf9b934105378dc1278"].into();
	let kono_oracle_account: AccountId = hex!["8ad57abb27cf9d6b067e8f8ec856600a71e20647884a52e10ad9e5af3f00013a"].into();
	rococo_parachain_runtime::GenesisConfig {
		frame_system: rococo_parachain_runtime::SystemConfig {
			code: rococo_parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: rococo_parachain_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		pallet_sudo: rococo_parachain_runtime::SudoConfig { key: root_key.clone() },
		parachain_info: rococo_parachain_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_aura: rococo_parachain_runtime::AuraConfig {
			authorities: initial_authorities,
		},
		cumulus_pallet_aura_ext: Default::default(),
		orml_tokens: rococo_parachain_runtime::TokensConfig {
			endowed_accounts: vec![
				// one is 1000_000_000_000
				(root_key.clone(), DOT, 1000_000_000_000_000_000_000),
				(root_key.clone(), BTC, 1000_000_000_000_000_000),
				(root_key.clone(), ETH, 1000_000_000_000_000_000_000_000),
				(get_account_id_from_seed::<sr25519::Public>("Alice"), DOT, 1000_000_000_000_000_000_000),
				(get_account_id_from_seed::<sr25519::Public>("Alice"), BTC, 1000_000_000_000_000_000),
				(get_account_id_from_seed::<sr25519::Public>("Alice"), ETH, 1000_000_000_000_000_000_000_000),
			]
		},
		pallet_floating_rate_lend: rococo_parachain_runtime::FloatingRateLendConfig {
			liquidation_threshold: FixedU128::from(1),
			pools: vec![
				(false, Vec::from("KONO".as_bytes()), KONO, root_key.clone()),
				(true, Vec::from("DOT".as_bytes()), DOT, root_key.clone()),
				(true, Vec::from("ETH".as_bytes()), ETH, root_key.clone()),
				(true, Vec::from("BTC".as_bytes()), BTC, root_key.clone()),
			]
		},
		pallet_chainlink_feed: rococo_parachain_runtime::ChainlinkFeedConfig {
			pallet_admin: Some(root_key.clone()),
			feed_creators: vec![root_key.clone()],
			feeds: vec![
				(
					root_key.clone(),
					1000000000 as u128,
					600,
					1,
					8,
					Vec::from("KONO / USD".as_bytes()),
					vec![
						(
							kono_oracle_account,
							root_key.clone(),
						)
					]
				),
				(
					root_key.clone(),
					1000000000 as u128,
					600,
					1,
					8,
					Vec::from("DOT / USD".as_bytes()),
					vec![
						(
							dot_oracle_account,
							root_key.clone(),
						)
					]
				),
				(
					root_key.clone(),
					1000000000 as u128,
					600,
					1,
					8,
					Vec::from("ETH / USD".as_bytes()),
					vec![
						(
							eth_oracle_account,
							root_key.clone(),
						)
					]
				),				(
					root_key.clone(),
					1000000000 as u128,
					600,
					1,
					8,
					Vec::from("BTC / USD".as_bytes()),
					vec![
						(
							btc_oracle_account,
							root_key.clone(),
						)
					]
				),
			]
		},
	}
}

fn shell_testnet_genesis(parachain_id: ParaId) -> shell_runtime::GenesisConfig {
	shell_runtime::GenesisConfig {
		frame_system: shell_runtime::SystemConfig {
			code: shell_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		parachain_info: shell_runtime::ParachainInfoConfig { parachain_id },
	}
}

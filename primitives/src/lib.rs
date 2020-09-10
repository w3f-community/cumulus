// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Cumulus related primitive types and traits.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
pub use xcm::{VersionedXcm, VersionedMultiAsset, VersionedMultiLocation};
use sp_runtime::traits::Block as BlockT;
pub use polkadot_core_primitives as relay_chain;
pub use polkadot_parachain::primitives::Id as ParaId;

pub mod validation_function_params;

/// Identifiers and types related to Cumulus Inherents
pub mod inherents {
	use sp_inherents::InherentIdentifier;

	/// Inherent identifier for downward messages.
	pub const DOWNWARD_MESSAGES_IDENTIFIER: InherentIdentifier = *b"cumdownm";

	/// The type of the inherent downward messages.
	pub type DownwardMessagesType = sp_std::vec::Vec<sp_std::vec::Vec<u8>>;

	/// The identifier for the `validation_function_params` inherent.
	pub const VALIDATION_FUNCTION_PARAMS_IDENTIFIER: InherentIdentifier = *b"valfunp0";
	/// The type of the inherent.
	pub type ValidationFunctionParamsType =
		crate::validation_function_params::ValidationFunctionParams;
}

/// Well known keys for values in the storage.
pub mod well_known_keys {
	/// The storage key for the upward messages.
	///
	/// The upward messages are stored as SCALE encoded `Vec<Vec<u8>>`.
	pub const UPWARD_MESSAGES: &'static [u8] = b":cumulus_upward_messages:";

	/// Current validation function parameters.
	pub const VALIDATION_FUNCTION_PARAMS: &'static [u8] = b":cumulus_validation_function_params";

	/// Code upgarde (set as appropriate by a pallet).
	pub const NEW_VALIDATION_CODE: &'static [u8] = b":cumulus_new_validation_code";

	/// The storage key for the processed downward messages.
	///
	/// The value is stored as SCALE encoded `u32`.
	pub const PROCESSED_DOWNWARD_MESSAGES: &'static [u8] = b":cumulus_processed_downward_messages:";
}

/// The head data of the parachain, stored in the relay chain.
#[derive(Decode, Encode, Debug)]
pub struct HeadData<Block: BlockT> {
	pub header: Block::Header,
}

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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

mod weights;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::{create_runtime_str, generic, impl_opaque_keys, traits::{BlakeTwo256, Block as BlockT, AccountIdLookup}, transaction_validity::{TransactionSource, TransactionValidity}, ApplyExtrinsicResult, FixedU128, FixedPointNumber};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types, match_type,
	traits::{Randomness, IsInVec, All},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	StorageValue,
};
use frame_system::limits::{BlockLength, BlockWeights};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

// XCM imports
use polkadot_parachain::primitives::Sibling;
use xcm::v0::{MultiAsset, MultiLocation, MultiLocation::*, Junction::*, BodyId, NetworkId};
use xcm_builder::{
	AccountId32Aliases, CurrencyAdapter, LocationInverter, ParentIsDefault, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SovereignSignedViaLocation, EnsureXcmOrigin, AllowUnpaidExecutionFrom, ParentAsSuperuser,
	AllowTopLevelPaidExecutionFrom, TakeWeightCredit, FixedWeightBounds,
	UsingComponents, SignedToAccountId32,
};
use xcm_executor::{Config, XcmExecutor};
use pallet_xcm::{XcmPassthrough, EnsureXcm, IsMajorityOfBody};
use xcm::v0::Xcm;

// Konomi required imports
use cumulus_primitives_core::ParaId;
use polkadot_parachain_primitives::*;
pub use pallet_currencies;
pub use pallet_floating_rate_lend;
use pallet_currencies::{BasicCurrencyAdapter, MultiCurrencyAdapter};
// use pallet_floating_rate_lend_rpc_runtime_api::{
// 	UserBalanceInfo as FloatingRateUserBalanceInfo,
// 	BalanceInfo as FloatingRateBalanceInfo,
// };

pub use orml_tokens;
use pallet_chainlink_oracle;
pub use pallet_chainlink_feed::RoundId;
use weights::chainlink::WeightInfo as ChainlinkWeightInfo;

use sp_runtime::{traits::{Convert, AccountIdConversion, Zero}};
use frame_support::{PalletId};
use orml_traits::GetByKey;
use sp_std::convert::TryInto;
use sp_runtime::traits::{CheckedConversion};
use sp_core::sp_std::convert::TryFrom;
use frame_support::traits::{Get, Contains};
use pallet_xcm_support::{XCMCurrencyAdapter};
use pallet_xcm_token;
use xcm_executor::traits::{MatchesFungible, Convert as XCMConvert, ShouldExecute, FilterAssetLocation};
use sp_std::borrow::Borrow;
// End of Konomi imports

pub type SessionHandlers = ();

// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
// pub type Balance = u128;
// pub type AssetId = u64;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("test-parachain"),
	impl_name: create_runtime_str!("test-parachain"),
	authoring_version: 1,
	spec_version: 14,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

pub const EPOCH_DURATION_IN_BLOCKS: u32 = 10 * MINUTES;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const ROC: Balance = 1_000_000_000_000;
pub const MILLIROC: Balance = 1_000_000_000;
pub const MICROROC: Balance = 1_000_000;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * MILLIROC;
	pub const TransferFee: u128 = 1 * MILLIROC;
	pub const CreationFee: u128 = 1 * MILLIROC;
	pub const TransactionByteFee: u128 = 1 * MICROROC;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Call = Call;
	type Event = Event;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const RocLocation: MultiLocation = X1(Parent);
	pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = X1(Parachain(ParachainInfo::parachain_id().into()));
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RococoNetwork, AccountId>,
);


/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	Currencies,
	// Use this currency when it is a fungible asset matching the given location or name:
	// IsConcrete<RocLocation>,
	KonomiIsConcrete,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RococoNetwork, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

parameter_types! {
	// One XCM operation is 1_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 1_000_000;
	// One ROC buys 1 second of weight.
	pub const WeightPrice: (MultiLocation, u128) = (X1(Parent), ROC);
}

match_type! {
	pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
		X1(Parent) | X2(Parent, Plurality { id: BodyId::Unit, .. })
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<All<MultiLocation>>,
	AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
	KonomiAllowedExecution,
	// ^^^ Parent & its unit plurality gets free execution
);


pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = XCMAssetTransactor; // LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = KonomiNativeAsset;
	type IsTeleporter = KonomiNativeAsset;	// <- should be enough to allow teleportation of ROC
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	type Trader = UsingComponents<IdentityFee<Balance>, RocLocation, AccountId, Currencies, ()>;
	type ResponseHandler = ();	// Don't handle responses for now.
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = (
	SignedToAccountId32<Origin, AccountId, RococoNetwork>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type XcmReserveTransferFilter = ();
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

impl cumulus_ping::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type Call = Call;
	type XcmSender = XcmRouter;
}

parameter_types! {
	pub const AssetDeposit: Balance = 1 * ROC;
	pub const ApprovalDeposit: Balance = 100 * MILLIROC;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 1 * ROC;
	pub const MetadataDepositPerByte: Balance = 10 * MILLIROC;
	pub const UnitBody: BodyId = BodyId::Unit;
}

/// A majority of the Unit body from Rococo over XCM is our required administration origin.
pub type AdminOrigin = EnsureXcm<IsMajorityOfBody<RocLocation, UnitBody>>;

impl pallet_assets::Config for Runtime {
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = AdminOrigin;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
}

// Konomi impls
pub struct KonomiNativeAsset;
impl FilterAssetLocation for KonomiNativeAsset {
	fn filter_asset_location(_asset: &MultiAsset, origin: &MultiLocation) -> bool {
		match origin {
			X2(Parent, Parachain(id)) => {
				match *id {
					18403 => true,
					_ => false
				}
			},
			_ => false
		}
	}
}

pub struct KonomiAllowedExecution;
impl ShouldExecute for KonomiAllowedExecution {
	fn should_execute<Call>(
		origin: &MultiLocation,
		_top_level: bool,
		_message: &Xcm<Call>,
		_shallow_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		match origin {
			X2(Parent, Parachain(id)) => {
				match *id {
					18403 => Ok(()),
					_ => Err(())
				}
			},
			_ => Err(())
		}
	}
}

pub struct AccountIdConvert;
impl XCMConvert<MultiLocation, AccountId> for AccountIdConvert {
	fn convert_ref(a: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		match a.borrow() {
			// TODO: should we keep X3?
			X3(_, Parachain(id), AccountId32{ network: _, id: account}) => {
				// TODO: separate into incoming and outgoing
				if ParaId::from(*id) != ParachainInfo::get() && *id != 18403 { return Err(()); }
				Ok(AccountId::from(*account))
			},
			X1(AccountId32{ network: _, id: account}) => {
				Ok(AccountId::from(*account))
			}
			_ => Err(())
		}
	}
}

/*pub type XCMAssetTransactor = XCMBalanceAdapter<
	Balances,
	KonomiIsConcrete,
	AccountId,
	AccountIdConvert,
	CurrencyId,
	MultiAssetToCurrencyIdConvert
>;*/

pub type XCMAssetTransactor = XCMCurrencyAdapter<
	Currencies,
	KonomiIsConcrete,
	AccountId,
	AccountIdConvert,
	CurrencyId,
	MultiAssetToCurrencyIdConvert
>;

pub struct MultiLocationToCurrencyIdConvert;
impl XCMConvert<MultiLocation, CurrencyId> for MultiLocationToCurrencyIdConvert {
	fn convert_ref(a: impl Borrow<MultiLocation>) -> Result<CurrencyId, ()> {
		match a.borrow() {
			X1(_) => Ok(DOT),
			X2(Parent, Parachain(id)) if ParaId::from(*id) == ParachainInfo::get() => Ok(KONO),
			X3(_, Parachain(id), GeneralIndex { id: key}) => {
				// TODO: this is for testing only. Separate into outgoing and incoming CurrencyConvertor
				if ParaId::from(*id) != ParachainInfo::get() && *id == 18401 {
					return Err(());
				}
				// decode the general key
				if let Some(currency_id) = CurrencyId::from_num(*key as u8) {
					match currency_id {
						DOT | KONO | BTC | ETH => Ok(currency_id),
						_ => Err(())
					}
				} else {
					Err(())
				}
			}
			_ => Err(())
		}
	}
}

pub struct MultiAssetToCurrencyIdConvert;
impl XCMConvert<MultiAsset, CurrencyId> for MultiAssetToCurrencyIdConvert {
	fn convert_ref(a: impl Borrow<MultiAsset>) -> Result<CurrencyId, ()> {
		match a.borrow() {
			MultiAsset::ConcreteFungible { id, amount: _} => {
				MultiLocationToCurrencyIdConvert::convert_ref(id)
			},
			_ => Err(()),
		}
	}
}

/// A `MatchesFungible` implementation. It matches concrete fungible assets
/// whose `id` could be converted into `CurrencyId`.
pub struct KonomiIsConcrete;
impl<Amount> MatchesFungible<Amount> for KonomiIsConcrete
	where
		Amount: TryFrom<u128>,
{
	fn matches_fungible(a: &MultiAsset) -> Option<Amount> {
		if let MultiAsset::ConcreteFungible { id, amount } = a {
			if MultiLocationToCurrencyIdConvert::convert_ref(id).is_ok() {
				return CheckedConversion::checked_from(*amount);
			}
		}
		None
	}
}

// Local Dependencies
parameter_types! {
	pub const GetBasicCurrencyId: CurrencyId = KONO;
}

pub struct IsCrossCurrencyId;
impl Contains<CurrencyId> for IsCrossCurrencyId {
	fn contains(t: &CurrencyId) -> bool {
		match t {
			CurrencyId::Cross(_) => true,
			_ => false
		}
	}
}

impl pallet_currencies::Config for Runtime {
	type Event = Event;
	type GetBasicCurrencyId = GetBasicCurrencyId;
	type IsCrossCurrencyId = IsCrossCurrencyId;
	type BasicCurrency = BasicCurrencyAdapter<Balances>;
	type MultiCurrency = MultiCurrencyAdapter<Tokens>;
	type CrossCurrency = CrossTokens;
}

pub struct ExistentialDeposits {}
impl GetByKey<CurrencyId, Balance> for ExistentialDeposits {
	fn get(_currency_id: &CurrencyId) -> Balance {
		Balance::zero()
	}
}

parameter_types! {
	pub const KonomiTreasuryPalletId: PalletId = PalletId(*b"kono/tsy");
}

parameter_types! {
	pub KonomiTreasuryAccount: AccountId = KonomiTreasuryPalletId::get().into_account();
}

/// Signed version of Balance
pub type Amount = i128;
// orml
impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, KonomiTreasuryAccount>;
	type MaxLocks = MaxLocks;
}

pub struct Conversion;

impl Convert<Balance, FixedU128> for Conversion {
	fn convert(a: Balance) -> FixedU128 {
		FixedU128::saturating_from_rational(a, BALANCE_ONE)
	}
}

impl Convert<FixedU128, Balance> for Conversion {
	fn convert(a: FixedU128) -> Balance {
		let accuracy = FixedU128::accuracy();
		(a.into_inner() / (accuracy / BALANCE_ONE)).try_into().unwrap()
	}
}

impl pallet_floating_rate_lend::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type PriceProvider = Oracle;
	type Conversion = Conversion;
}

pub type FeedId = u32;
pub type Value = u128;

parameter_types! {
	pub const FeedPalletId: PalletId = PalletId(*b"linkfeed");
	pub const MinimumReserve: Balance = ExistentialDeposit::get() * 1000;
	pub const OracleCountLimit: u32 = 25;
	pub const FeedLimit: FeedId = 100;
}

impl pallet_chainlink_feed::Config for Runtime {
	type Event = Event;
	type FeedId = FeedId;
	type Value = Value;
	type Currency = Balances;
	type PalletId = FeedPalletId;
	type MinimumReserve = MinimumReserve;
	type StringLimit = StringLimit;
	type OnAnswerHandler = ();
	type OracleCountLimit = OracleCountLimit;
	type FeedLimit = FeedLimit;
	type WeightInfo = ChainlinkWeightInfo;
}

parameter_types! {
	pub const UnsignedInterval: BlockNumber = 4;
	pub const UnsignedPriority: u32 = 5;
}

pub struct CurrencyToFeedIdConverter;
impl Convert<CurrencyId, Option<FeedId>> for CurrencyToFeedIdConverter {
	fn convert(a: CurrencyId) -> Option<FeedId> {
		match a {
			KONO => Some(FeedId::from(0 as u8)),
			DOT => Some(FeedId::from(1 as u8)),
			ETH => Some(FeedId::from(2 as u8)),
			BTC => Some(FeedId::from(3 as u8)),
			_ => None
		}
	}
}

impl pallet_chainlink_oracle::Config for Runtime {
	type Oracle = ChainlinkFeed;
	type CurrencyFeedConvertor = CurrencyToFeedIdConverter;
}

pub struct XCMAssetConverter;
impl Convert<(CurrencyId, Balance), Option<MultiAsset>> for XCMAssetConverter {
	fn convert(a: (CurrencyId, u128)) -> Option<MultiAsset> {
		let id = match a.0 {
			DOT => Some(X3(Parent, Parachain(ParachainInfo::get().into()), GeneralIndex { id: NATIVE_DOT_INDEX as u128 })),
			_ => None
		};
		if id.is_none() { return None; }
		Some(MultiAsset::ConcreteFungible{id: id.unwrap(), amount: a.1})
	}
}

pub struct XCMSelfLocConverter;
impl Convert<AccountId, MultiLocation> for XCMSelfLocConverter {
	fn convert(a: AccountId) -> MultiLocation {
		X3(Parent, Parachain(ParachainInfo::get().into()), AccountId32 { network: NetworkId::Polkadot, id: a.into() })
	}
}

pub struct XCMAccountConverter;
impl Convert<(ParachainId, AccountId), Option<MultiLocation>> for XCMAccountConverter {
	fn convert(a: (ParachainId, AccountId)) -> Option<MultiLocation> {
		match a.0 {
			ParachainId::KonomiTestChain => Some(X1(AccountId32 { network: NetworkId::Polkadot, id: a.1.into() })),
			_ => None
		}
	}
}

pub struct XCMDestinationConverter;
impl Convert<ParachainId, Option<MultiLocation>> for XCMDestinationConverter {
	fn convert(a: ParachainId) -> Option<MultiLocation> {
		match a {
			// TODO: this is just for testing, here we have the testing statemint chain, 18401
			ParachainId::KonomiTestChain => Some(X2(Parent, Parachain(18401))),
			_ => None
		}
	}
}

impl pallet_xcm_token::Config for Runtime {
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	type XCMAssetConverter = XCMAssetConverter;
	type XCMSelfLocConverter = XCMSelfLocConverter;
	type XCMAccountConverter = XCMAccountConverter;
	type XCMDestinationConverter = XCMDestinationConverter;
	type MultiCurrency = Currencies;
}
// End of konomi impls

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},

		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>, ValidateUnsigned} = 20,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 21,

		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 30,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 31,

		Aura: pallet_aura::{Pallet, Config<T>},
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config},

		CrossTokens: pallet_xcm_token::{Pallet, Storage},
		Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>} = 11,
		Currencies: pallet_currencies::{Pallet, Call, Storage, Event<T>},
		Oracle: pallet_chainlink_oracle::{Pallet, Call, Storage},
		ChainlinkFeed: pallet_chainlink_feed::{Pallet, Call, Storage, Config<T>, Event<T>} = 55,
		FloatingRateLend: pallet_floating_rate_lend::{Pallet, Call, Storage, Config<T>, Event<T>} = 15,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 50,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 51,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 52,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 53,

		Spambot: cumulus_ping::{Pallet, Call, Storage, Event<T>} = 99,
	}
}

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = sp_runtime::MultiSignature;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;
/// Balance of an account.
pub type Balance = u128;
/// Index of a transaction in the chain.
pub type Index = u32;
/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// An index to a block.
pub type BlockNumber = u32;
/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
		}
	}

	// impl pallet_floating_rate_lend_rpc_runtime_api::LendingApi<Block, PoolId, FixedU128, AccountId> for Runtime {
    //     fn supply_rate(id: PoolId) -> FixedU128 {
    //         FloatingRateLend::supply_rate(id)
	// 	}
	//
	// 	fn debt_rate(id: PoolId) -> FixedU128 {
    //         FloatingRateLend::debt_rate(id)
	// 	}
	//
	// 	fn user_balances(user: AccountId) -> FloatingRateUserBalanceInfo<FixedU128> {
	// 		match FloatingRateLend::user_balances(user) {
	// 			Ok(user_balances) => FloatingRateUserBalanceInfo{
	// 				total_supply: user_balances.supply_balance,
	// 				borrow_limit: user_balances.collateral_balance,
	// 				total_borrow: user_balances.debt_balance,
	// 			},
	// 			Err(_) => FloatingRateUserBalanceInfo{
	// 				total_supply: FixedU128::zero(),
	// 				borrow_limit: FixedU128::zero(),
	// 				total_borrow: FixedU128::zero(),
	// 			},
	// 		}
	// 	}
	//
	// 	fn user_debt_balance(pool_id: PoolId, user: AccountId) -> FloatingRateBalanceInfo<FixedU128> {
	// 		let amount = FloatingRateLend::user_debt_balance(pool_id, user).unwrap_or_else(|_| FixedU128::zero());
	// 		FloatingRateBalanceInfo{amount}
	// 	}
	//
	// 	fn user_supply_balance(pool_id: PoolId, user: AccountId) -> FloatingRateBalanceInfo<FixedU128> {
	// 		let amount = FloatingRateLend::user_supply_balance(pool_id, user).unwrap_or_else(|_| FixedU128::zero());
	// 		FloatingRateBalanceInfo{amount}
	// 	}
    // }	// impl pallet_floating_rate_lend_rpc_runtime_api::LendingApi<Block, PoolId, FixedU128, AccountId> for Runtime {
    //     fn supply_rate(id: PoolId) -> FixedU128 {
    //         FloatingRateLend::supply_rate(id)
	// 	}
	//
	// 	fn debt_rate(id: PoolId) -> FixedU128 {
    //         FloatingRateLend::debt_rate(id)
	// 	}
	//
	// 	fn user_balances(user: AccountId) -> FloatingRateUserBalanceInfo<FixedU128> {
	// 		match FloatingRateLend::user_balances(user) {
	// 			Ok(user_balances) => FloatingRateUserBalanceInfo{
	// 				total_supply: user_balances.supply_balance,
	// 				borrow_limit: user_balances.collateral_balance,
	// 				total_borrow: user_balances.debt_balance,
	// 			},
	// 			Err(_) => FloatingRateUserBalanceInfo{
	// 				total_supply: FixedU128::zero(),
	// 				borrow_limit: FixedU128::zero(),
	// 				total_borrow: FixedU128::zero(),
	// 			},
	// 		}
	// 	}
	//
	// 	fn user_debt_balance(pool_id: PoolId, user: AccountId) -> FloatingRateBalanceInfo<FixedU128> {
	// 		let amount = FloatingRateLend::user_debt_balance(pool_id, user).unwrap_or_else(|_| FixedU128::zero());
	// 		FloatingRateBalanceInfo{amount}
	// 	}
	//
	// 	fn user_supply_balance(pool_id: PoolId, user: AccountId) -> FloatingRateBalanceInfo<FixedU128> {
	// 		let amount = FloatingRateLend::user_supply_balance(pool_id, user).unwrap_or_else(|_| FixedU128::zero());
	// 		FloatingRateBalanceInfo{amount}
	// 	}
    // }
}

cumulus_pallet_parachain_system::register_validate_block!(
	Runtime,
	cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
);

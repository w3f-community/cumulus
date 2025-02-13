use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug, FixedU128, FixedPointNumber};
use frame_system::Config;
use sp_std::{vec::Vec};
use frame_support::sp_runtime::traits::Zero;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/* ---------- Price ------------ */
pub type PriceValue = FixedU128;

/// The price type used by all Pallets.
/// Please note that this struct is IMMUTABLE.
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub struct Price<T> where T: Config {
    price: PriceValue,
    updated_at: <T as Config>::BlockNumber,
}

impl <T> Price<T> where T: Config {
    /// Create a new price struct instance
    pub fn new(val: PriceValue, block_number: <T as Config>::BlockNumber) -> Price<T> {
        Price {
            price: val,
            updated_at: block_number,
        }
    }

    pub fn invalid_price() -> Self {
        Price {
            price: PriceValue::zero(),
            updated_at: <T as Config>::BlockNumber::zero(),
        }
    }

    /// Checks if the price is ready to be used
    /// TODO: check the current block number is more than a certain
    /// TODO: threshold of the last updated block number
    pub fn price_ready(&self) -> bool {
        self.price.is_positive()
    }

    /// Get the value of the price
    pub fn value(&self) -> PriceValue {
        self.price
    }
}

/* ---------- errors Related ----------- */
pub type PoolId = u64;

/* ---------- Currency Related ------------ */
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Basic {
    pub id: u8,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Native {
    pub id: u8,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Cross {
    pub id: u8,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Erc20 {
    pub id: u8,
}

/// The representation of a currency in a multi-currency system
#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
    /// Basic
    Basic(Basic),
    /// Native on chain multi-currency
    Native(Native),
    /// Currency on other chain
    Cross(Cross),
    /// ERC20 tokens
    Erc20(Erc20),
}

impl CurrencyId {
    pub fn is_native_currency(&self) -> bool {
        matches!(self, CurrencyId::Native {..})
    }

    pub fn is_erc20_currency(&self) -> bool {
        matches!(self, CurrencyId::Erc20 {..})
    }

    pub fn is_basic_currency(&self) -> bool {
        matches!(self, CurrencyId::Basic {..})
    }

    // TODO: refactor this part, this is for testing
    pub fn decode(bits: Vec<u8>) -> Option<Self> {
        match bits[0] {
            48 => Some(CurrencyId::Basic(Basic { id: bits[1]-48})),
            49 => Some(CurrencyId::Native(Native { id: bits[1]-48})),
            _ => None
        }
    }

    // TODO: refactor this part, this is for testing
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            0 => Some(CurrencyId::Basic(Basic { id: 0})),
            NATIVE_DOT_INDEX => Some(DOT),
            2 => Some(CurrencyId::Native(Native { id: 1})),
            3 => Some(CurrencyId::Native(Native { id: 2})),
            CROSS_DOT_INDEX => Some(CROSS_DOT),
            _ => None
        }
    }
}

pub const KONO: CurrencyId = CurrencyId::Basic(Basic { id: 0});
pub const DOT: CurrencyId = CurrencyId::Native(Native { id: 0});
pub const NATIVE_DOT_INDEX: u8 = 1;
pub const ETH: CurrencyId = CurrencyId::Native(Native { id: 1});
pub const BTC: CurrencyId = CurrencyId::Native(Native { id: 2});

pub const CROSS_DOT: CurrencyId = CurrencyId::Cross(Cross { id: 0});
pub const CROSS_DOT_INDEX: u8 = 4;

// TODO: maybe each currency will have their own decimal
pub const BALANCE_ONE: u128 = u128::pow(10, 12);

#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ParachainId {
    /// The statemint chain
    Statemint,
    /// The konomi test chain
    KonomiTestChain
}
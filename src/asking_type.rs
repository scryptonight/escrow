use scrypto::prelude::*;

use crate::util::length_of_option_set;

/// This is a general way of describing how many tokens are wanted as
/// payment for some trade. Its purpose is to enable us to specify
/// *either* "n amount of fungibles" *or* "these here non-fungibles"
/// depending on which type of resource it is paired with.
#[derive(ScryptoSbor, ManifestSbor, Clone, PartialEq, Eq, Debug)]
pub enum AskingType {
    /// Asks for this exact amount of a fungible token.
    Fungible(Decimal),

    /// Asks for non-fungible tokens. We can ask for a specific set of
    /// non-fungible local ids and/or we can ask for a number of
    /// arbitrarily chosen NFTs. If both parameters are in use then we
    /// are asking for the sum of those two so for example,
    /// ```NonFungible(Some(IndexSet(1,2,3)), Some(5))``` asks for
    /// nonfungible local ids 1, 2 and 3 PLUS also 5 other random NFTs
    /// of the same NFT resource. (And by random we mean arbitrary.)
    NonFungible(Option<IndexSet<NonFungibleLocalId>>, Option<u64>),
}

impl AskingType {
    /// Determines how many tokens in total are being asked for in
    /// an instance of the `AskingType` enum.
    pub fn to_amount(&self) -> Decimal {
        match self {
            AskingType::Fungible(price) => price.clone(),
            AskingType::NonFungible(set, amount)
                => Decimal::from(amount.unwrap_or_default()
                                 + length_of_option_set(set) as u64),
        }
    }

    /// Checks if the values in the provided map are consistent
    /// and make sense. Panics if this does not hold.
    pub fn check_asking_map_sanity(map: &IndexMap<ResourceAddress, AskingType>) {
        for (resaddr, ask) in map {
            let fung_res = resaddr.is_fungible();
            match ask {
                AskingType::Fungible(amount) => {
                    assert!(fung_res, "fungible AskingType used for non-fungible resource");
                    assert!(!amount.is_negative(), "cannot ask for negative amounts");
                },
                AskingType::NonFungible(_, _) => {
                    assert!(!fung_res, "non-fungible AskingType used for fungible resource");
                }
            }
        }
    }
}

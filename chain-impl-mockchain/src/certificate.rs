use crate::block::Message;
use crate::key::*;
use crate::stake::{StakeKeyId, StakePoolId};
use chain_core::property;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeKeyRegistration {
    pub stake_key_id: StakeKeyId,
}

impl StakeKeyRegistration {
    pub fn make_certificate(self, stake_private_key: &PrivateKey) -> Message {
        Message::StakeKeyRegistration(Signed {
            sig: stake_private_key.serialize_and_sign(&self),
            data: self,
        })
    }
}

impl property::Serialize for StakeKeyRegistration {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(writer);
        self.stake_key_id.serialize(&mut codec)?;
        Ok(())
    }
}

impl property::Deserialize for StakeKeyRegistration {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(reader);
        Ok(StakeKeyRegistration {
            stake_key_id: StakeKeyId::deserialize(&mut codec)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeKeyDeregistration {
    pub stake_key_id: StakeKeyId,
}

impl StakeKeyDeregistration {
    pub fn make_certificate(self, stake_private_key: &PrivateKey) -> Message {
        Message::StakeKeyDeregistration(Signed {
            sig: stake_private_key.serialize_and_sign(&self),
            data: self,
        })
    }
}

impl property::Serialize for StakeKeyDeregistration {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(writer);
        self.stake_key_id.serialize(&mut codec)?;
        Ok(())
    }
}

impl property::Deserialize for StakeKeyDeregistration {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(reader);
        Ok(StakeKeyDeregistration {
            stake_key_id: StakeKeyId::deserialize(&mut codec)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeDelegation {
    pub stake_key_id: StakeKeyId,
    pub pool_id: StakePoolId,
}

impl StakeDelegation {
    pub fn make_certificate(self, stake_private_key: &PrivateKey) -> Message {
        // FIXME: "It must be signed by sks_source, and that key must
        // be included in the witness." - why?
        Message::StakeDelegation(Signed {
            sig: stake_private_key.serialize_and_sign(&self),
            data: self,
        })
    }
}

impl property::Serialize for StakeDelegation {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(writer);
        self.stake_key_id.serialize(&mut codec)?;
        self.pool_id.serialize(&mut codec)?;
        Ok(())
    }
}

impl property::Deserialize for StakeDelegation {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(reader);
        Ok(StakeDelegation {
            stake_key_id: StakeKeyId::deserialize(&mut codec)?,
            pool_id: StakePoolId::deserialize(&mut codec)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakePoolRegistration {
    pub pool_id: StakePoolId,
    //pub owner: StakeKeyId, // FIXME: support list of owners
    // reward sharing params: cost, margin, pledged amount of stake
    // alternative stake key reward account
}

impl StakePoolRegistration {
    /// Create a certificate for this stake pool registration, signed
    /// by the pool's staking key and the owners.
    pub fn make_certificate(self, pool_private_key: &PrivateKey) -> Message {
        Message::StakePoolRegistration(Signed {
            sig: pool_private_key.serialize_and_sign(&self),
            data: self,
        })
    }
}

impl property::Serialize for StakePoolRegistration {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(writer);
        self.pool_id.serialize(&mut codec)?;
        //self.owner.serialize(&mut codec)?;
        Ok(())
    }
}

impl property::Deserialize for StakePoolRegistration {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(reader);
        Ok(StakePoolRegistration {
            pool_id: StakePoolId::deserialize(&mut codec)?,
            // owner: StakeKeyId::deserialize(&mut codec)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakePoolRetirement {
    pub pool_id: StakePoolId,
    // TODO: add epoch when the retirement will take effect
}

impl StakePoolRetirement {
    /// Create a certificate for this stake pool retirement, signed
    /// by the pool's staking key.
    pub fn make_certificate(self, pool_private_key: &PrivateKey) -> Message {
        Message::StakePoolRetirement(Signed {
            sig: pool_private_key.serialize_and_sign(&self),
            data: self,
        })
    }
}

impl property::Serialize for StakePoolRetirement {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(writer);
        self.pool_id.serialize(&mut codec)?;
        Ok(())
    }
}

impl property::Deserialize for StakePoolRetirement {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::from(reader);
        Ok(StakePoolRetirement {
            pool_id: StakePoolId::deserialize(&mut codec)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl<T: Arbitrary> Arbitrary for Signed<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Signed {
                data: Arbitrary::arbitrary(g),
                sig: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for StakeKeyRegistration {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            StakeKeyRegistration {
                stake_key_id: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for StakeKeyDeregistration {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            StakeKeyDeregistration {
                stake_key_id: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for StakeDelegation {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            StakeDelegation {
                stake_key_id: Arbitrary::arbitrary(g),
                pool_id: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for StakePoolRegistration {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            StakePoolRegistration {
                pool_id: Arbitrary::arbitrary(g),
                //owner: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for StakePoolRetirement {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            StakePoolRetirement {
                pool_id: Arbitrary::arbitrary(g),
            }
        }
    }
}
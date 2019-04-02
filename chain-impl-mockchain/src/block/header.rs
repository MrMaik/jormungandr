use crate::block::{
    headerraw::HeaderRaw,
    version::{AnyBlockVersion, BlockVersion},
};
use crate::date::BlockDate;
use crate::key::{
    deserialize_public_key, deserialize_signature, serialize_public_key, serialize_signature,
    verify_signature, Hash,
};
use crate::leadership::{bft, genesis};
use chain_core::{
    mempack::{read_from_raw, ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::{
    self, Curve25519_2HashDH, Ed25519Extended, FakeMMM, Signature, VerifiableRandomFunction,
    Verification,
};

pub type HeaderHash = Hash;
pub type BlockContentHash = Hash;
pub type BlockId = Hash;
pub type BlockContentSize = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Common {
    pub any_block_version: AnyBlockVersion,
    pub block_date: BlockDate,
    pub block_content_size: BlockContentSize,
    pub block_content_hash: BlockContentHash,
    pub block_parent_hash: BlockId,
    pub chain_length: ChainLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainLength(pub(crate) u32);

pub type HeaderToSign = Common;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BftProof {
    pub(crate) leader_id: bft::LeaderId,
    pub(crate) signature: BftSignature,
}

#[derive(Debug, Clone)]
pub struct BftSignature(pub(crate) Signature<HeaderToSign, Ed25519Extended>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisPraosProof {
    pub(crate) genesis_praos_id: genesis::GenesisPraosId,
    pub(crate) vrf_proof: <Curve25519_2HashDH as VerifiableRandomFunction>::VerifiedRandom,
    pub(crate) kes_proof: KESSignature,
}

#[derive(Debug, Clone)]
pub struct KESSignature(pub(crate) Signature<HeaderToSign, FakeMMM>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    /// In case there is no need for consensus layer and no need for proof of the
    /// block. This may apply to the genesis block for example.
    None,
    Bft(BftProof),
    GenesisPraos(GenesisPraosProof),
}

/// this is the block header, it contains the necessary data
/// to prove a given block has been signed by the appropriate
/// nodes, it also contains the metadata to localize the block
/// within the blockchain (the block date and the parent's hash)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub(crate) common: Common,
    pub(crate) proof: Proof,
}

impl PartialEq<Self> for BftSignature {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}
impl Eq for BftSignature {}

impl PartialEq<Self> for KESSignature {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}
impl Eq for KESSignature {}

/*
impl Proof {
    pub fn leader_id(&self) -> Option<LeaderId> {
        match self {
            Proof::None => None,
            Proof::Bft(bft_proof) => Some(LeaderId::Bft(bft_proof.leader_id.clone())),
            Proof::GenesisPraos(genesis_praos_proof) => {
                Some(LeaderId::GenesisPraos(GenesisPraosLeader {
                    kes_public_key: genesis_praos_proof.kes_public_key.clone(),
                    vrf_public_key: genesis_praos_proof.vrf_public_key.clone(),
                }))
            }
        }
    }
}
*/

impl Header {
    #[inline]
    pub fn block_version(&self) -> AnyBlockVersion {
        self.common.any_block_version
    }

    #[inline]
    pub fn block_date(&self) -> &BlockDate {
        &self.common.block_date
    }

    #[inline]
    pub fn block_content_hash(&self) -> &BlockContentHash {
        &self.common.block_content_hash
    }

    #[inline]
    pub fn block_parent_hash(&self) -> &BlockId {
        &self.common.block_parent_hash
    }

    pub fn chain_length(&self) -> ChainLength {
        self.common.chain_length
    }

    pub fn to_raw(&self) -> Result<HeaderRaw, std::io::Error> {
        use chain_core::property::Serialize;
        self.serialize_as_vec().map(HeaderRaw)
    }

    /// function to compute the Header Hash as per the spec. It is the hash
    /// of the serialized header (except the first 2bytes: the size)
    #[inline]
    pub fn hash(&self) -> HeaderHash {
        // TODO: this is not the optimal way to compute the hash
        use chain_core::property::Serialize;
        let bytes = self.serialize_as_vec().unwrap();
        HeaderHash::hash_bytes(&bytes[..])
    }

    pub fn proof(&self) -> &Proof {
        &self.proof
    }

    /// this function verify the proof and the consistency of the block
    /// within itself.
    pub fn verify_proof(&self) -> Verification {
        match &self.proof {
            Proof::None => Verification::Success,
            Proof::Bft(bft_proof) => {
                verify_signature(&bft_proof.signature.0, &bft_proof.leader_id.0, &self.common)
            }
            Proof::GenesisPraos(genesis_praos_proof) => {
                let _kes_public_key = {
                    // use the ID to find the expected keys
                    let _id = &genesis_praos_proof.genesis_praos_id;
                    unimplemented!()
                };
                /*
                verify_signature(
                    &genesis_praos_proof.kes_proof.0,
                    &kes_public_key,
                    &self.common,
                )
                */
                // TODO: verify the VRF too
            }
        }
    }
}

impl property::ChainLength for ChainLength {
    fn next(&self) -> Self {
        ChainLength(self.0 + 1)
    }
}

impl property::Serialize for Common {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::Codec;
        use std::io::Write;

        let mut codec = Codec::from(writer);

        codec.put_u16(self.any_block_version.into())?;
        codec.put_u32(self.block_content_size)?;
        codec.put_u32(self.block_date.epoch)?;
        codec.put_u32(self.block_date.slot_id)?;
        codec.put_u32(self.chain_length.0)?;
        codec.write_all(self.block_content_hash.as_ref())?;
        codec.write_all(self.block_parent_hash.as_ref())?;

        Ok(())
    }
}

impl property::Serialize for Header {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        self.common.serialize(&mut writer)?;

        match &self.proof {
            Proof::None => {}
            Proof::Bft(bft_proof) => {
                serialize_public_key(&bft_proof.leader_id.0, &mut writer)?;
                serialize_signature(&bft_proof.signature.0, &mut writer)?;
            }
            Proof::GenesisPraos(genesis_praos_proof) => {
                genesis_praos_proof
                    .genesis_praos_id
                    .serialize(&mut writer)?;
                {
                    let mut buf =
                        [0; <Curve25519_2HashDH as VerifiableRandomFunction>::VERIFIED_RANDOM_SIZE];
                    genesis_praos_proof.vrf_proof.to_bytes(&mut buf);
                    writer.write_all(&buf)?;
                }
                serialize_signature(&genesis_praos_proof.kes_proof.0, writer)?;
            }
        }
        Ok(())
    }
}

impl Readable for Common {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let any_block_version = buf.get_u16().map(Into::into)?;
        let block_content_size = buf.get_u32()?;
        let epoch = buf.get_u32()?;
        let slot_id = buf.get_u32()?;
        let chain_length = buf.get_u32().map(ChainLength)?;
        let block_content_hash = Hash::read(buf)?;
        let block_parent_hash = Hash::read(buf)?;

        let block_date = BlockDate { epoch, slot_id };
        Ok(Common {
            any_block_version,
            block_content_size,
            block_date,
            chain_length,
            block_content_hash,
            block_parent_hash,
        })
    }
}

impl Readable for Header {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let common = Common::read(buf)?;

        let proof = match common.any_block_version {
            AnyBlockVersion::Supported(BlockVersion::Genesis) => Proof::None,
            AnyBlockVersion::Supported(BlockVersion::Ed25519Signed) => {
                // BFT
                let leader_id = deserialize_public_key(buf).map(bft::LeaderId)?;
                let signature = deserialize_signature(buf).map(BftSignature)?;
                Proof::Bft(BftProof {
                    leader_id,
                    signature,
                })
            }
            AnyBlockVersion::Supported(BlockVersion::KesVrfproof) => {
                let genesis_praos_id = genesis::GenesisPraosId::read(buf)?;
                dbg!(&genesis_praos_id);
                let vrf_proof = {
                    let bytes = <[u8;<Curve25519_2HashDH as VerifiableRandomFunction>::VERIFIED_RANDOM_SIZE]>::read(buf)?;

                    <Curve25519_2HashDH as VerifiableRandomFunction>::VerifiedRandom::from_bytes_unverified(&bytes)
                        .ok_or(ReadError::StructureInvalid("VRF Proof".to_string()))
                }?;
                dbg!(&vrf_proof);
                let kes_proof = deserialize_signature(buf).map(KESSignature)?;
                dbg!(&kes_proof);

                Proof::GenesisPraos(GenesisPraosProof {
                    genesis_praos_id: genesis_praos_id,
                    vrf_proof: vrf_proof,
                    kes_proof: kes_proof,
                })
            }
            AnyBlockVersion::Unsupported(version) => {
                return Err(ReadError::UnknownTag(version as u32));
            }
        };

        Ok(Header { common, proof })
    }
}

impl property::Deserialize for Header {
    type Error = std::io::Error;
    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        let raw = HeaderRaw::deserialize(reader)?;
        read_from_raw(raw.as_ref())
    }
}

impl property::Header for Header {
    type Id = HeaderHash;
    type Date = BlockDate;
    type Version = AnyBlockVersion;
    type ChainLength = ChainLength;

    fn id(&self) -> Self::Id {
        self.hash()
    }
    fn parent_id(&self) -> Self::Id {
        self.block_parent_hash().clone()
    }
    fn chain_length(&self) -> Self::ChainLength {
        self.common.chain_length
    }
    fn date(&self) -> Self::Date {
        *self.block_date()
    }
    fn version(&self) -> Self::Version {
        self.block_version()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::block::ConsensusVersion;
    use chain_crypto::AsymmetricKey;
    use num_traits::FromPrimitive;
    use quickcheck::{Arbitrary, Gen, TestResult};

    quickcheck! {
        fn header_serialization_bijection(b: Header) -> TestResult {
            property::testing::serialization_bijection_r(b)
        }
    }

    impl Arbitrary for AnyBlockVersion {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            AnyBlockVersion::from(u16::arbitrary(g) % 3)
        }
    }

    impl Arbitrary for ConsensusVersion {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            ConsensusVersion::from_u16(u16::arbitrary(g) % 3).unwrap()
        }
    }

    impl Arbitrary for Common {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Common {
                any_block_version: Arbitrary::arbitrary(g),
                block_date: Arbitrary::arbitrary(g),
                block_content_size: Arbitrary::arbitrary(g),
                block_content_hash: Arbitrary::arbitrary(g),
                block_parent_hash: Arbitrary::arbitrary(g),
                chain_length: ChainLength(Arbitrary::arbitrary(g)),
            }
        }
    }

    impl Arbitrary for BftProof {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let sk = crate::key::test::arbitrary_secret_key(g);
            let pk = sk.to_public();
            let signature = chain_crypto::Signature::generate(&sk, &[0u8, 1, 2, 3]);
            BftProof {
                leader_id: bft::LeaderId(pk),
                signature: BftSignature(signature.coerce()),
            }
        }
    }
    impl Arbitrary for GenesisPraosProof {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            use rand_chacha::ChaChaRng;
            use rand_core::SeedableRng;
            let mut seed = [0; 32];
            for byte in seed.iter_mut() {
                *byte = Arbitrary::arbitrary(g);
            }
            let mut rng = ChaChaRng::from_seed(seed);

            let genesis_praos_id = genesis::GenesisPraosId(Arbitrary::arbitrary(g));

            let vrf_proof = {
                let sk = Curve25519_2HashDH::generate(&mut rng);
                Curve25519_2HashDH::evaluate(&sk, &[0, 1, 2, 3], &mut rng)
            };

            let kes_proof = {
                let mut sk = crate::key::test::arbitrary_secret_key(g);
                let signature = Signature::generate_update(&mut sk, &[0u8, 1, 2, 3]);
                KESSignature(signature)
            };
            GenesisPraosProof {
                genesis_praos_id: genesis_praos_id,
                vrf_proof: vrf_proof,
                kes_proof: kes_proof,
            }
        }
    }

    impl Arbitrary for Header {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let common = Common::arbitrary(g);
            let proof = match common.any_block_version {
                AnyBlockVersion::Supported(BlockVersion::Genesis) => Proof::None,
                AnyBlockVersion::Supported(BlockVersion::Ed25519Signed) => {
                    Proof::Bft(Arbitrary::arbitrary(g))
                }
                AnyBlockVersion::Supported(BlockVersion::KesVrfproof) => {
                    Proof::GenesisPraos(Arbitrary::arbitrary(g))
                }
                AnyBlockVersion::Unsupported(_) => unreachable!(),
            };
            Header {
                common: common,
                proof: proof,
            }
        }
    }
}
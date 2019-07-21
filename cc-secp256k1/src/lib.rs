use cc::{CryptoError, Hash, PrivateKey, PublicKey, Signature};
use secp256k1::{Message, Secp256k1, SignOnly, ThirtyTwoByteHash, VerifyOnly};

use std::convert::TryFrom;

pub struct Secp256k1PrivateKey {
    secret_key: secp256k1::SecretKey,
    engine: Secp256k1<SignOnly>,
}

pub struct Secp256k1PublicKey {
    pub_key: secp256k1::PublicKey,
    engine: Secp256k1<VerifyOnly>,
}

pub struct Secp256k1Signature {
    sig: secp256k1::Signature,
    engine: Secp256k1<VerifyOnly>,
}

#[derive(Debug, PartialEq)]
pub struct Secp256k1Error(secp256k1::Error);

pub struct HashedMessage<'a>(&'a Hash);

//
// PrivateKey Impl
//

impl TryFrom<&[u8]> for Secp256k1PrivateKey {
    type Error = CryptoError;

    fn try_from(bytes: &[u8]) -> Result<Secp256k1PrivateKey, Self::Error> {
        let secret_key = secp256k1::SecretKey::from_slice(bytes).map_err(Secp256k1Error)?;
        let engine = Secp256k1::signing_only();

        Ok(Secp256k1PrivateKey { secret_key, engine })
    }
}

impl PrivateKey<32> for Secp256k1PrivateKey {
    type PublicKey = Secp256k1PublicKey;
    type Signature = Secp256k1Signature;

    fn sign_message(&self, msg: &Hash) -> Self::Signature {
        let msg = Message::from(HashedMessage(msg));
        let sig = self.engine.sign(&msg, &self.secret_key);
        let engine = Secp256k1::verification_only();

        Secp256k1Signature { sig, engine }
    }

    fn pub_key(&self) -> Self::PublicKey {
        let pub_key = secp256k1::PublicKey::from_secret_key(&self.engine, &self.secret_key);
        let engine = Secp256k1::verification_only();

        Secp256k1PublicKey { pub_key, engine }
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.secret_key[..]);

        bytes
    }
}

//
// PublicKey Impl
//

impl TryFrom<&[u8]> for Secp256k1PublicKey {
    type Error = CryptoError;

    fn try_from(bytes: &[u8]) -> Result<Secp256k1PublicKey, Self::Error> {
        let pub_key = secp256k1::PublicKey::from_slice(bytes).map_err(Secp256k1Error)?;
        let engine = Secp256k1::verification_only();

        Ok(Secp256k1PublicKey { pub_key, engine })
    }
}

impl PublicKey<33> for Secp256k1PublicKey {
    type Signature = Secp256k1Signature;

    fn verify_signature(&self, msg: &Hash, sig: &Self::Signature) -> Result<(), CryptoError> {
        let msg = Message::from(HashedMessage(msg));

        self.engine
            .verify(&msg, &sig.sig, &self.pub_key)
            .map_err(Secp256k1Error)?;

        Ok(())
    }

    fn to_bytes(&self) -> [u8; 33] {
        self.pub_key.serialize()
    }
}

//
// Signature Impl
//

impl TryFrom<&[u8]> for Secp256k1Signature {
    type Error = CryptoError;

    fn try_from(bytes: &[u8]) -> Result<Secp256k1Signature, Self::Error> {
        let sig = secp256k1::Signature::from_compact(bytes).map_err(Secp256k1Error)?;
        let engine = Secp256k1::verification_only();

        Ok(Secp256k1Signature { sig, engine })
    }
}

impl Signature<64> for Secp256k1Signature {
    type PublicKey = Secp256k1PublicKey;

    fn verify(&self, msg: &Hash, pub_key: &Self::PublicKey) -> Result<(), CryptoError> {
        let msg = Message::from(HashedMessage(msg));

        self.engine
            .verify(&msg, &self.sig, &pub_key.pub_key)
            .map_err(Secp256k1Error)?;

        Ok(())
    }

    fn to_bytes(&self) -> [u8; 64] {
        self.sig.serialize_compact()
    }
}

//
// Error Impl
//

impl From<Secp256k1Error> for CryptoError {
    fn from(err: Secp256k1Error) -> Self {
        use secp256k1::Error;

        match err.0 {
            Error::IncorrectSignature => CryptoError::InvalidSignature,
            Error::InvalidMessage => CryptoError::InvalidLength,
            Error::InvalidPublicKey => CryptoError::InvalidPublicKey,
            Error::InvalidSignature => CryptoError::InvalidSignature,
            Error::InvalidSecretKey => CryptoError::InvalidPrivateKey,
            Error::InvalidRecoveryId => CryptoError::InvalidSignature,
            Error::InvalidTweak => CryptoError::Other("secp256k1: bad tweak"),
            Error::NotEnoughMemory => CryptoError::Other("secp256k1: not enough memory"),
        }
    }
}

//
// HashedMessage Impl
//

impl<'a> HashedMessage<'a> {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl<'a> ThirtyTwoByteHash for HashedMessage<'a> {
    fn into_32(self) -> [u8; 32] {
        self.to_bytes()
    }
}
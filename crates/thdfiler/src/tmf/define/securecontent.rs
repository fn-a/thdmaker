//! Secure Content extension.
//!
//! This module implements the 3MF Secure Content Extension specification which
//! provides encryption and access control capabilities for 3MF files containing
//! sensitive or proprietary 3D model data.

use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use base64::prelude::*;
use super::error::{Error, Result};

/// The algorithm to use for encryption.
pub enum Algorithm {
    RsaOaepMgf1p,
    // Digest method and mask generation function.
    RsaOaepCouple(String, String),
}

impl Algorithm {
    /// Use SHA1 as the digest method and MGF1 as the mask generation function.
    /// *Older* but widely supported algorithm.
    pub const RSA_OAEP_MGF1P: &str = "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p";
    /// Allows specifying different [digest methods](from digestmethod attribute value) and [mask generation functions](from mgfalgorithm attribute value).
    /// *Recommended* for new implementations, default use MGF1 with SHA1.
    pub const RSA_OAEP: &str = "http://www.w3.org/2001/04/xmlenc#rsa-oaep";
    /// MGF1 with SHA1
    pub const MGF1_SHA1: &str = "http://www.w3.org/2009/xmlenc11#mgf1sha1";
    /// MGF1 with SHA224
    pub const MGF1_SHA224: &str = "http://www.w3.org/2009/xmlenc11#mgf1sha224";
    /// MGF1 with SHA256
    pub const MGF1_SHA256: &str = "http://www.w3.org/2009/xmlenc11#mgf1sha256";
    /// MGF1 with SHA384
    pub const MGF1_SHA384: &str = "http://www.w3.org/2009/xmlenc11#mgf1sha384";
    /// MGF1 with SHA512
    pub const MGF1_SHA512: &str = "http://www.w3.org/2009/xmlenc11#mgf1sha512";
    /// SHA1 algorithm
    pub const SHA1: &str = "http://www.w3.org/2000/09/xmldsig#sha1";
    /// SHA256 algorithm
    pub const SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
    /// SHA384 algorithm
    pub const SHA384: &str = "http://www.w3.org/2001/04/xmlenc#sha384";
    /// SHA512 algorithm
    pub const SHA512: &str = "http://www.w3.org/2001/04/xmlenc#sha512";
    /// AES256-GCM algorithm is an authenticated encryption mechanism.
    /// It is equivalent to doing these two operations in one step - AES encryption followed by HMAC signing.
    pub const AES_GCM: &str = "http://www.w3.org/2009/xmlenc11#aes256-gcm";
}

/// The Key Store part consists of a keystore element that encapsulates encryption key data and references to the encrypted content.
#[derive(Debug, Clone, Default)]
pub struct KeyStore {
    /// A universally unique ID that allows the Key Store to be identified.
    pub uuid: Uuid,
    /// A set of consumer elements.
    pub consumers: Vec<Consumer>,
    /// A set of resource data group elements.
    pub resource_data_groups: Vec<ResourceDataGroup>,
}

impl KeyStore {
    /// Create a new KeyStore with a random UUID.
    pub fn new(uuid: &str) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            consumers: Vec::new(),
            resource_data_groups: Vec::new(),
        })
    }
}

/// The consumer element under a keystore element contains the target consumer specific information.
#[derive(Debug, Clone, Default)]
pub struct Consumer {
    /// ID of the target consumer.
    pub consumer_id: String,
    /// Optional key identifier.
    pub key_id: Option<String>,
    /// Optional public key value in PEM format.
    pub key_value: Option<KeyValue>,
}

impl Consumer {
    /// Create a new Consumer.
    pub fn new(consumer_id: impl Into<String>) -> Self {
        Self {
            consumer_id: consumer_id.into(),
            key_id: None,
            key_value: None,
        }
    }
}

/// The keyvalue element wrap the Encryption Key string.
#[derive(Debug, Clone, Default)]
pub struct KeyValue(pub String);

/// The resourcedatagroup element under a keystore element contains the resource specific encryption information for a set of encrypted resources which are encrypted using the same Content Encryption Key (CEK).
#[derive(Debug, Clone, Default)]
pub struct ResourceDataGroup {
    /// UUID that identifies the CEK.
    pub key_uuid: Uuid,
    /// Access rights for consumers.
    pub access_rights: Vec<AccessRight>,
    /// Encrypted resources.
    pub resource_datas: Vec<ResourceData>,
}

impl ResourceDataGroup {
    /// Create a new ResourceDataGroup with a random UUID.
    pub fn new(uuid: &str) -> Result<Self> {
        Ok(Self {
            key_uuid: Uuid::parse_str(uuid)?,
            access_rights: Vec::new(),
            resource_datas: Vec::new(),
        })
    }
}

/// The accessright element under a resourcedata element contains the consumer specific information to unwrap the Content Encryption Keys for a specific consumer.
#[derive(Debug, Clone, Default)]
pub struct AccessRight {
    /// Zero-based index to the consumer element containing the keys to unwrap the Content Encryption Keys.
    pub consumer_index: u32,
    /// Wrapping method used to wrap the CEK using a Key Encryption Key (KEK).
    pub kek_params: KEKParams,
    /// The CEK to decrypt the content file, which is wrapped for a specific consumer using the KEK.
    pub cipher_data: CipherData,
}

impl AccessRight {
    /// Create a new AccessRight.
    pub fn new(consumer_index: u32, kek_params: KEKParams, cipher_data: CipherData) -> Self {
        Self {
            consumer_index,
            kek_params,
            cipher_data,
        }
    }
}

/// The kekparams element under a accessright element specifies the wrapping method used to wrap the CEK using a Key Encryption Key (KEK).
#[derive(Debug, Clone)]
pub struct KEKParams {
    /// Wrapping algorithm used to wrap the Content Encryption Key (CEK).
    pub wrapping_algorithm: String,
    /// Mask generation function used to wrap the Content Encryption Key (CEK).
    pub mgf_algorithm: Option<String>,
    /// Message digest method used to wrap the Content Encryption Key (CEK).
    pub digest_method: Option<String>,
}

impl Default for KEKParams {
    fn default() -> Self {
        Self {
            wrapping_algorithm: Algorithm::RSA_OAEP_MGF1P.to_string(),
            mgf_algorithm: None,
            digest_method: None,
        }
    }
}

/// The base64 bit data wrapped.
#[derive(Debug, Clone, Default)]
pub struct Base64Value(pub Vec<u8>);

impl FromStr for Base64Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(BASE64_STANDARD.decode(s)?))
    }
}

impl fmt::Display for Base64Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE64_STANDARD.encode(&self.0))
    }
}

/// The cipherdata element under the accessright element contains the CEK to decrypt the content file.
#[derive(Debug, Clone, Default)]
pub struct CipherData {
    /// The encrypted key payload.
    pub cipher_value: CipherValue,
}

impl CipherData {
    /// Create a new CipherData.
    pub fn new() -> Self {
        Self { cipher_value: CipherValue::default() }
    }

    /// Set the cipher value from base64-encoded string.
    pub fn set_value(&mut self, value: &str) -> Result<&mut Self> {
        self.cipher_value.0 = value.parse()?;
        Ok(self)
    }

    /// Get the cipher value as base64-encoded string.
    pub fn get_value(&self) -> String {
        self.cipher_value.0.to_string()
    }
}

/// The content as base64-encoded string wrapped in xenc:CipherValue element.
#[derive(Debug, Clone, Default)]
pub struct CipherValue(pub Base64Value);

/// The resourcedata element under a resourcedatagroup element contains the resource specific encryption information for an encrypted resource and the file path to the encrypted content file.
#[derive(Debug, Clone, Default)]
pub struct ResourceData {
    /// Path to the encrypted resource file.
    pub path: String,
    /// CEK specific encryption information.
    pub cek_params: CEKParams,
}

impl ResourceData {
    /// Create a new ResourceData.
    pub fn new(path: impl Into<String>, cek_params: CEKParams) -> Self {
        Self {
            path: path.into(),
            cek_params,
        }
    }
}

/// The cekparams element under a resourcedata element contains the CEK specific encryption information for an encrypted resource.
#[derive(Debug, Clone)]
pub struct CEKParams {
    /// Encryption algorithm used to encrypt the resource data.
    pub encryption_algorithm: String,
    /// Compression algorithm applied to content before encryption.
    pub compression: Compression,
    /// The Initialization Vector.
    pub iv: Option<Base64Value>,
    /// The Authentication Tag.
    pub tag: Option<Base64Value>,
    /// The Additional Authenticated Data.
    pub aad: Option<Base64Value>,
}

impl Default for CEKParams {
    fn default() -> Self {
        Self {
            encryption_algorithm: Algorithm::AES_GCM.to_string(),
            compression: Compression::None,
            iv: None,
            tag: None,
            aad: None,
        }
    }
}

impl CEKParams {
    /// Set the Initialization Vector from base64-encoded string.
    pub fn set_iv(&mut self, iv: &str) -> Result<&mut Self> {
        self.iv = Some(iv.parse()?);
        Ok(self)
    }

    /// Set the Authentication Tag from base64-encoded string.
    pub fn set_tag(&mut self, tag: &str) -> Result<&mut Self> {
        self.tag = Some(tag.parse()?);
        Ok(self)
    }

    /// Set the Additional Authenticated Data from base64-encoded string.
    pub fn set_aad(&mut self, aad: &str) -> Result<&mut Self> {
        self.aad = Some(aad.parse()?);
        Ok(self)
    }

    /// Get the Initialization Vector as base64-encoded string.
    pub fn get_iv(&self) -> Option<String> {
        self.iv.as_ref().map(|iv| iv.to_string())
    }

    /// Get the Authentication Tag as base64-encoded string.
    pub fn get_tag(&self) -> Option<String> {
        self.tag.as_ref().map(|tag| tag.to_string())
    }

    /// Get the Additional Authenticated Data as base64-encoded string.
    pub fn get_aad(&self) -> Option<String> {
        self.aad.as_ref().map(|aad| aad.to_string())
    }
}

/// Compression algorithm applied to content before encryption.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Compression {
    #[default]
    None,
    Deflate,
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compression::None => write!(f, "none"),
            Compression::Deflate => write!(f, "deflate"),
        }
    }
}

impl FromStr for Compression {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Compression::None),
            "deflate" => Ok(Compression::Deflate),
            _ => Err(Error::InvalidAttribute {
                name: "compression".to_string(),
                message: format!("unknown compression: {}", s),
            }),
        }
    }
}
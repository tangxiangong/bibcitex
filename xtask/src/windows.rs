//! Signed Velopack metadata. The signed bytes retain architecture and localized notes.
use crate::{Result, ensure, safe_file, versions};
use base64::{Engine, engine::general_purpose::STANDARD};
use p256::{
    ecdsa::{
        Signature, SigningKey, VerifyingKey,
        signature::{Signer, Verifier},
    },
    pkcs8::{DecodePrivateKey, DecodePublicKey},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
struct Envelope {
    feed: String,
    signature: String,
}

fn validate(feed: &[u8], arch: &str, version: &str) -> Result<Value> {
    let value: Value = serde_json::from_slice(feed)?;
    ensure(
        value["Architecture"] == arch,
        "Signed Windows architecture mismatch",
    )?;
    let assets = value["Assets"].as_array().ok_or("Invalid Windows feed")?;
    ensure(assets.len() == 1, "Expected one signed Windows package")?;
    let asset = &assets[0];
    let channel = versions(&format!("v{version}"))?.channel;
    ensure(
        asset["PackageId"] == "BibCiTeX" && asset["Version"] == version,
        "Signed Windows identity mismatch",
    )?;
    ensure(
        asset["Type"] == "Full" || asset["Type"] == 1,
        "Expected full Windows package",
    )?;
    ensure(
        asset["FileName"] == format!("BibCiTeX-{version}-win-{arch}-{channel}-full.nupkg"),
        "Signed Windows filename mismatch",
    )?;
    ensure(
        asset["Size"].as_u64().is_some_and(|size| size > 0),
        "Invalid Windows package size",
    )?;
    ensure(
        asset["SHA256"]
            .as_str()
            .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())),
        "Invalid Windows package SHA-256",
    )?;
    Ok(value)
}

fn seal(feed: &[u8], key: &SigningKey) -> Result<Vec<u8>> {
    let signature: Signature = key.sign(feed);
    Ok(serde_json::to_vec(&Envelope {
        feed: STANDARD.encode(feed),
        signature: STANDARD.encode(signature.to_bytes()),
    })?)
}

/// Sign both architectures using the release environment's PKCS#8 key.
pub fn sign(directory: &Path, tag: &str, public_key: &str) -> Result<()> {
    let version = versions(tag)?;
    let private_key = STANDARD.decode(std::env::var("WINDOWS_UPDATE_PRIVATE_KEY")?.trim())?;
    let signing = SigningKey::from_pkcs8_der(&private_key)?;
    sign_with(
        directory,
        &version.version,
        &version.channel,
        public_key,
        &signing,
    )
}

fn sign_with(
    directory: &Path,
    version: &str,
    channel: &str,
    public_key: &str,
    signing: &SigningKey,
) -> Result<()> {
    let verifying = VerifyingKey::from_public_key_der(&STANDARD.decode(public_key.trim())?)?;
    ensure(
        signing.verifying_key() == &verifying,
        "Windows update private key does not match the shipped public key",
    )?;
    // Validate both architectures before writing either envelope.
    let mut envelopes = Vec::new();
    for arch in ["x64", "arm64"] {
        let name = format!("releases.win-{arch}-{channel}");
        let mut feed: Value =
            serde_json::from_slice(&fs::read(safe_file(directory, &format!("{name}.json"))?)?)?;
        feed["Architecture"] = arch.into();
        let bytes = serde_json::to_vec(&feed)?;
        validate(&bytes, arch, version)?;
        envelopes.push((format!("{name}.signed.json"), seal(&bytes, signing)?));
    }
    for (name, bytes) in envelopes {
        fs::write(directory.join(name), bytes)?;
    }
    Ok(())
}

/// Verify the envelope and require the same assets as the legacy feed.
pub fn verify(
    directory: &Path,
    arch: &str,
    version: &str,
    channel: &str,
    public_key: &str,
) -> Result<()> {
    let name = format!("releases.win-{arch}-{channel}");
    let envelope: Envelope = serde_json::from_slice(&fs::read(safe_file(
        directory,
        &format!("{name}.signed.json"),
    )?)?)?;
    let feed = STANDARD.decode(envelope.feed)?;
    let signature = Signature::from_slice(&STANDARD.decode(envelope.signature)?)?;
    let key = VerifyingKey::from_public_key_der(&STANDARD.decode(public_key.trim())?)?;
    key.verify(&feed, &signature)
        .map_err(|_| "Invalid Windows update signature")?;
    let signed = validate(&feed, arch, version)?;
    let plain: Value =
        serde_json::from_slice(&fs::read(safe_file(directory, &format!("{name}.json"))?)?)?;
    ensure(
        signed["Assets"] == plain["Assets"],
        "Signed and legacy Windows feeds differ",
    )
}

#[cfg(test)]
pub(crate) fn test_sign(directory: &Path, version: &str, channel: &str) -> String {
    use p256::pkcs8::EncodePublicKey;
    let signing = SigningKey::from_bytes((&[23; 32]).into()).unwrap();
    let public = STANDARD.encode(
        signing
            .verifying_key()
            .to_public_key_der()
            .unwrap()
            .as_bytes(),
    );
    sign_with(directory, version, channel, &public, &signing).unwrap();
    public
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tampered_metadata_and_wrong_architecture_are_rejected() {
        let (directory, keys) = crate::tests::fixture("v1.0.0-beta.1");
        verify(
            directory.path(),
            "x64",
            "1.0.0-beta.1",
            "beta",
            &keys.windows,
        )
        .unwrap();
        let name = "releases.win-x64-beta.signed.json";
        fs::copy(
            directory.path().join(name),
            directory.path().join("releases.win-arm64-beta.signed.json"),
        )
        .unwrap();
        assert!(
            verify(
                directory.path(),
                "arm64",
                "1.0.0-beta.1",
                "beta",
                &keys.windows
            )
            .is_err()
        );
        let mut envelope: Envelope =
            serde_json::from_slice(&fs::read(directory.path().join(name)).unwrap()).unwrap();
        let mut bytes = STANDARD.decode(&envelope.feed).unwrap();
        bytes[0] ^= 1;
        envelope.feed = STANDARD.encode(bytes);
        fs::write(
            directory.path().join(name),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        assert!(
            verify(
                directory.path(),
                "x64",
                "1.0.0-beta.1",
                "beta",
                &keys.windows
            )
            .is_err()
        );
    }
}

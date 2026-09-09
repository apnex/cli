use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;

struct DuplicateKeyProbe;

impl<'de> Deserialize<'de> for DuplicateKeyProbe {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        struct ObjectVisitor;
        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = DuplicateKeyProbe;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an object with unique decoded keys")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut seen = BTreeSet::new();
                while let Some((key, _value)) = map.next_entry::<String, Box<RawValue>>()? {
                    if !seen.insert(key) {
                        return Err(de::Error::custom("Duplicate decoded object key"));
                    }
                }
                Ok(DuplicateKeyProbe)
            }
        }
        input.deserialize_map(ObjectVisitor)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for token in ["9007199254740993", "18446744073709551616", "1e400", "-0", "1.2300", "1E+04"] {
        let raw: Box<RawValue> = serde_json::from_str(token)?;
        assert_eq!(raw.get(), token);
        assert_eq!(serde_json::to_string(&raw)?, token);
    }
    for token in ["NaN", "Infinity", "01", "+1", "1e", "1."] {
        assert!(serde_json::from_str::<Box<RawValue>>(token).is_err());
    }
    assert!(serde_json::from_str::<String>(r#""\uD800""#).is_err());
    assert!(serde_json::from_str::<DuplicateKeyProbe>(r#"{"name":0,"n\u0061me":1}"#).is_err());
    assert!(serde_json::from_str::<DuplicateKeyProbe>(r#"{"0":0,"":1}"#).is_ok());

    let root = std::env::temp_dir().join(format!("cli-authoring-fs-probe-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root)?;
    let lock = OpenOptions::new().write(true).create_new(true).open(root.join("session.lock"))?;
    lock.try_lock()?;
    let second = OpenOptions::new().write(true).open(root.join("session.lock"))?;
    assert!(matches!(second.try_lock(), Err(std::fs::TryLockError::WouldBlock)));

    let state = root.join("session.json");
    fs::write(&state, b"old")?;
    let temporary = root.join("next.json");
    let mut output = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    output.write_all(b"new")?;
    output.sync_all()?;
    fs::rename(&temporary, &state)?;
    File::open(&root)?.sync_all()?;
    assert_eq!(fs::read(&state)?, b"new");

    let export = root.join("export.json");
    fs::hard_link(&state, &export)?;
    assert!(fs::hard_link(&state, &export).is_err());
    assert_eq!(fs::read(&export)?, b"new");
    assert_eq!(format!("{:x}", Sha256::digest(b"abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let _editor = reedline::Reedline::create();
    drop(second);
    drop(lock);
    fs::remove_dir_all(root)?;
    println!("PASS: exact number tokens, invalid tokens, decoded duplicate keys, Unicode rejection, local lock contention, replace and sync, create-only publication, SHA-256, and editor construction");
    Ok(())
}

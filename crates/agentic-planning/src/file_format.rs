use crate::{error::Result, Error, PlanningEngine};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const APLAN_MAGIC: [u8; 4] = *b"PLAN";
pub const APLAN_VERSION: u16 = 1;
#[allow(dead_code)]
pub const APLAN_HEADER_SIZE: usize = 128;
#[allow(dead_code)]
pub const APLAN_FOOTER_SIZE: usize = 64;
pub const APLAN_INTEGRITY_MARKER: [u8; 8] = *b"PLANEND\0";

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AplanHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u32,
    pub created_at: i64,
    pub modified_at: i64,
    pub goal_count: u32,
    pub decision_count: u32,
    pub commitment_count: u32,
    pub dream_count: u32,
    pub federation_count: u32,
    pub goal_section_offset: u64,
    pub decision_section_offset: u64,
    pub commitment_section_offset: u64,
    pub dream_section_offset: u64,
    pub federation_section_offset: u64,
    pub index_section_offset: u64,
    pub checksum: [u8; 32],
    pub reserved: [u8; 14],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AplanFooter {
    pub file_size: u64,
    pub write_count: u64,
    pub last_session: [u8; 16],
    pub integrity: [u8; 8],
    pub footer_checksum: [u8; 16],
    pub reserved: [u8; 8],
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedPlan {
    header: AplanHeader,
    goals: std::collections::HashMap<crate::GoalId, crate::Goal>,
    decisions: std::collections::HashMap<crate::DecisionId, crate::Decision>,
    commitments: std::collections::HashMap<crate::CommitmentId, crate::Commitment>,
    dreams: std::collections::HashMap<crate::DreamId, crate::Dream>,
    federations: std::collections::HashMap<crate::FederationId, crate::Federation>,
    soul_archive: std::collections::HashMap<crate::GoalId, crate::GoalSoulArchive>,
    indexes: crate::PlanIndexes,
    footer: AplanFooter,
}

impl PlanningEngine {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let tmp_path = temp_path_for(path);

        // R4: Crash recovery — if .aplan.tmp exists but .aplan does not, recover
        if !path.exists() && tmp_path.exists() {
            std::fs::rename(&tmp_path, path)?;
        }
        // If both exist, use .aplan (the completed write); clean up stale tmp
        if path.exists() && tmp_path.exists() {
            let _ = std::fs::remove_file(&tmp_path);
        }

        if path.exists() {
            Self::load(path)
        } else {
            Self::create(path)
        }
    }

    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let mut engine = Self::in_memory();
        engine.path = Some(path.as_ref().to_path_buf());
        engine.save()?;
        Ok(engine)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let bytes = std::fs::read(path_ref)?;
        let persisted: PersistedPlan = serde_json::from_slice(&bytes)?;

        if persisted.header.magic != APLAN_MAGIC || persisted.header.version != APLAN_VERSION {
            return Err(Error::InvalidFile);
        }
        if persisted.footer.integrity != APLAN_INTEGRITY_MARKER {
            return Err(Error::InvalidFile);
        }

        // R2: Checksum verification — use BTreeMap for deterministic serialization order
        let is_legacy = persisted.header.checksum == [0u8; 32];
        if !is_legacy {
            let sorted_goals: BTreeMap<_, _> = persisted.goals.iter().collect();
            let sorted_decisions: BTreeMap<_, _> = persisted.decisions.iter().collect();
            let sorted_commitments: BTreeMap<_, _> = persisted.commitments.iter().collect();
            let sorted_dreams: BTreeMap<_, _> = persisted.dreams.iter().collect();
            let sorted_federations: BTreeMap<_, _> = persisted.federations.iter().collect();
            let sorted_souls: BTreeMap<_, _> = persisted.soul_archive.iter().collect();

            let encoded_state = serde_json::to_vec(&(
                &sorted_goals,
                &sorted_decisions,
                &sorted_commitments,
                &sorted_dreams,
                &sorted_federations,
                &sorted_souls,
                &persisted.indexes,
            ))?;
            let computed = blake3::hash(&encoded_state);
            if *computed.as_bytes() != persisted.header.checksum {
                return Err(Error::CorruptedFile(format!(
                    "checksum mismatch: expected {:?}, computed {:?}",
                    &persisted.header.checksum[..4],
                    &computed.as_bytes()[..4]
                )));
            }
        }

        let mut engine = Self::in_memory();
        engine.path = Some(path_ref.to_path_buf());
        engine.goal_store = persisted.goals;
        engine.decision_store = persisted.decisions;
        engine.commitment_store = persisted.commitments;
        engine.dream_store = persisted.dreams;
        engine.federation_store = persisted.federations;
        engine.soul_archive = persisted.soul_archive;
        engine.indexes = persisted.indexes;
        engine.write_count = persisted.footer.write_count;
        engine.dirty = false;
        Ok(engine)
    }

    pub fn save(&mut self) -> Result<()> {
        let Some(path) = self.path.clone() else {
            return Ok(());
        };

        // R5: File-level locking — prevent concurrent writes to the same .aplan file
        let _file_lock = crate::locking::FileLock::acquire(&path).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                format!("failed to acquire file lock: {e}"),
            ))
        })?;

        let now = crate::Timestamp::now().0;

        // R1: Increment write_count
        self.write_count += 1;

        // R2: Use BTreeMap for deterministic serialization order (HashMap is non-deterministic)
        let sorted_goals: BTreeMap<_, _> = self.goal_store.iter().collect();
        let sorted_decisions: BTreeMap<_, _> = self.decision_store.iter().collect();
        let sorted_commitments: BTreeMap<_, _> = self.commitment_store.iter().collect();
        let sorted_dreams: BTreeMap<_, _> = self.dream_store.iter().collect();
        let sorted_federations: BTreeMap<_, _> = self.federation_store.iter().collect();
        let sorted_souls: BTreeMap<_, _> = self.soul_archive.iter().collect();

        let encoded_state = serde_json::to_vec(&(
            &sorted_goals,
            &sorted_decisions,
            &sorted_commitments,
            &sorted_dreams,
            &sorted_federations,
            &sorted_souls,
            &self.indexes,
        ))?;

        // R1: Compute real blake3 checksum of payload
        let digest = blake3::hash(&encoded_state);

        // R3: Compute section offsets (estimated from serialization order)
        // These are approximate — computed from cumulative serialized sizes
        let goal_bytes = serde_json::to_vec(&sorted_goals)?;
        let decision_bytes = serde_json::to_vec(&sorted_decisions)?;
        let commitment_bytes = serde_json::to_vec(&sorted_commitments)?;
        let dream_bytes = serde_json::to_vec(&sorted_dreams)?;
        let federation_bytes = serde_json::to_vec(&sorted_federations)?;

        let mut offset: u64 = 0;
        let goal_offset = offset;
        offset += goal_bytes.len() as u64;
        let decision_offset = offset;
        offset += decision_bytes.len() as u64;
        let commitment_offset = offset;
        offset += commitment_bytes.len() as u64;
        let dream_offset = offset;
        offset += dream_bytes.len() as u64;
        let federation_offset = offset;
        offset += federation_bytes.len() as u64;
        let index_offset = offset;

        let header = AplanHeader {
            magic: APLAN_MAGIC,
            version: APLAN_VERSION,
            flags: 0,
            created_at: now,
            modified_at: now,
            goal_count: self.goal_store.len() as u32,
            decision_count: self.decision_store.len() as u32,
            commitment_count: self.commitment_store.len() as u32,
            dream_count: self.dream_store.len() as u32,
            federation_count: self.federation_store.len() as u32,
            goal_section_offset: goal_offset,
            decision_section_offset: decision_offset,
            commitment_section_offset: commitment_offset,
            dream_section_offset: dream_offset,
            federation_section_offset: federation_offset,
            index_section_offset: index_offset,
            checksum: *digest.as_bytes(),
            reserved: [0; 14],
        };

        // R1: Set last_session from engine session state and compute footer checksum
        let session_bytes = *self.session_id.as_bytes();
        let footer_payload = [
            &(encoded_state.len() as u64).to_le_bytes()[..],
            &self.write_count.to_le_bytes(),
            &session_bytes,
        ]
        .concat();
        let footer_hash = blake3::hash(&footer_payload);

        let footer = AplanFooter {
            file_size: encoded_state.len() as u64,
            write_count: self.write_count,
            last_session: session_bytes,
            integrity: APLAN_INTEGRITY_MARKER,
            footer_checksum: footer_hash.as_bytes()[..16].try_into().unwrap_or([0; 16]),
            reserved: [0; 8],
        };

        let persisted = PersistedPlan {
            header,
            goals: self.goal_store.clone(),
            decisions: self.decision_store.clone(),
            commitments: self.commitment_store.clone(),
            dreams: self.dream_store.clone(),
            federations: self.federation_store.clone(),
            soul_archive: self.soul_archive.clone(),
            indexes: self.indexes.clone(),
            footer,
        };

        let data = serde_json::to_vec_pretty(&persisted)?;

        // R4: Atomic write — write to temp, fsync, rename
        let temp_path = temp_path_for(&path);
        let temp_file = std::fs::File::create(&temp_path)?;
        use std::io::Write;
        let mut writer = std::io::BufWriter::new(temp_file);
        writer.write_all(&data)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        drop(writer);

        std::fs::rename(&temp_path, &path)?;

        self.dirty = false;
        Ok(())
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut temp = path.to_path_buf();
    temp.set_extension("aplan.tmp");
    temp
}

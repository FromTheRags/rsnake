use crate::game_logic::logger::error_formatting::LocatedError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use snafu::{Location, ResultExt, Snafu, ensure};
use std::path::{Path, PathBuf};
use tracing::{debug, info};
// Equivalent contextual error with thiserror (illustration only; no dependency added):
//
// #[derive(Debug, thiserror::Error)]
// enum DbError {
//     #[error("Could not open database {}", path.display())]
//     Open {
//         path: PathBuf,
//         source: sled::Error,
//         location: &'static std::panic::Location<'static>,
//     },
// }
// fn open(path: &Path) -> std::result::Result<Db, DbError> {
//     sled::open(path).map_err(|source| DbError::Open {
//         path: path.to_path_buf(),
//         source,
//         location: std::panic::Location::caller(),
//     })
// }
//
// thiserror supports #[from] for source-only variants, but path/location context
// needs explicit construction here: https://docs.rs/thiserror/latest/thiserror/.
// Snafu's with_context builds the variant and #[snafu(implicit)] captures the
// file/line/column at that context call, rather than the later logging call.
// Location references static metadata: no stack walk or symbol resolution.
// Together with the source chain, it pinpoints our failing DB operation with
// less overhead and noise than a backtrace for these expected I/O failures.
// A backtrace remains useful when the complete runtime call path matters;
// location only identifies where we attached context, not internals of sled.
// Display contains only the message. format_error_chain reads location() from
// each LocatedError in the source chain and prints it alongside the message.
// Third-party sources without location metadata retain their message.
//
/// Database failures retain their underlying source for tracing and diagnostics.
#[derive(Debug, Snafu)]
pub enum HighScoreError {
    #[snafu(display("Could not open high-score database {}", path.display()))]
    OpenDatabase {
        path: PathBuf,
        source: sled::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Could not generate a score ID in {}", path.display()))]
    GenerateId {
        path: PathBuf,
        source: sled::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Could not read entry {index} from {}", path.display()))]
    ReadEntry {
        path: PathBuf,
        index: usize,
        source: sled::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Could not save score {score} at key {key:02x?} in {}", path.display()
    ))]
    SaveScore {
        path: PathBuf,
        key: Vec<u8>,
        score: u32,
        source: sled::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Could not trim {} to {limit} scores", path.display()))]
    TrimDatabase {
        path: PathBuf,
        limit: usize,
        source: sled::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Could not flush high-score database {} to disk", path.display()
    ))]
    FlushDatabase {
        path: PathBuf,
        source: sled::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Could not encode score {score} recorded at {date}"))]
    Encode {
        score: u32,
        date: DateTime<Utc>,
        source: toml::ser::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Invalid UTF-8 at key {key:02x?} in {}: {data:02x?}", path.display()
    ))]
    Utf8 {
        path: PathBuf,
        key: Vec<u8>,
        data: Vec<u8>,
        source: std::str::Utf8Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Invalid TOML at key {key:02x?} in {}: {data:?}", path.display()
    ))]
    Decode {
        path: PathBuf,
        key: Vec<u8>,
        data: String,
        #[snafu(source(from(toml::de::Error, Box::new)))]
        source: Box<toml::de::Error>,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Invalid score key {key:02x?} in {}: expected at least 4 bytes", path.display()
    ))]
    InvalidKey {
        path: PathBuf,
        key: Vec<u8>,
        #[snafu(implicit)]
        location: Location,
    },
}

impl LocatedError for HighScoreError {
    fn location(&self) -> Option<Location> {
        Some(match self {
            Self::OpenDatabase { location, .. }
            | Self::GenerateId { location, .. }
            | Self::ReadEntry { location, .. }
            | Self::SaveScore { location, .. }
            | Self::TrimDatabase { location, .. }
            | Self::FlushDatabase { location, .. }
            | Self::Encode { location, .. }
            | Self::Utf8 { location, .. }
            | Self::Decode { location, .. }
            | Self::InvalidKey { location, .. } => *location,
        })
    }
    // All causes here are external errors: the formatter follows Error::source().
}

pub type Result<T> = std::result::Result<T, HighScoreError>;

const MAX_SCORE_ENTRIES: usize = 10;
const DB_FILE: &str = "high_scores.db";
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HighScore {
    pub symbols: String,
    pub score: u32,
    pub speed: String,
    // snake_growth_factor is a new field that we added to the HighScore struct,
    // add a default value for backward compatibility
    #[serde(default = "default_snake_growth_factor")]
    pub snake_growth_factor: u16,
    pub date: DateTime<Utc>,
    pub version: String,
    // Keep the same sled ID across retries; it is not part of the stored score.
    #[serde(skip)]
    database_id: Option<u64>,
}
impl HighScore {
    #[must_use]
    pub fn new(symbols: String, score: u32, speed: String, snake_growth_factor: u16) -> Self {
        HighScore {
            symbols,
            score,
            speed,
            snake_growth_factor,
            date: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_id: None,
        }
    }
}
/// Add a default value (in function because of serde) to adapt for older entries in db without `snake_growth` without breaking
const fn default_snake_growth_factor() -> u16 {
    1
}
/// # Motivation for this DB
/// To use something else than SQL DB to change ;) Top-edge DB
/// For a more rock solid DB in Rust use redb (more typed (no manuel BE management), more stable, less innovant)
/// # Strengths
/// - Sled database guarantees that its iterators, including those returned by `db.iter()` and `db.range()`,
///   will return elements in lexicographical order of their keys. (as raw byte slices)
/// - This is a fundamental feature of sled because it is built upon a $\text{Bw-Tree}$ structure, which is a type of ordered, persistent tree.
/// - The sorting is strictly lexicographical; you must ensure that multibyte numeric keys are stored in Big-Endian byte order.
/// - Big-Endian (BE): Stores the Most Significant Byte (MSB) first. This is required for correct lexicographical sorting of numbers.
/// # To explore the DB content by hands:
/// - cargo install sledcli (or use EDMA)
/// - hex or str to change the view
/// - Pairs or keys
/// # Why TOML serialisation
/// - Provide a better visualization by hands
/// - Not too much data, TOML overhead is unseen
/// - Already used for argument serialisation, avoid a new dependency
///   (and the bincode crate story is a lesson-teller)
/// - NB: on other projects with others constrains postcard,rkyv,or borsh
pub struct HighScoreManager {
    db: Db,
    path: PathBuf,
}

impl HighScoreManager {
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn new() -> Result<Self> {
        let manager = HighScoreManager::new_with_custom_path(DB_FILE)?;
        Ok(manager)
    }
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn new_with_custom_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        debug!(path = %path.display(), "Opening high-score database");
        let db = sled::open(&path).with_context(|_| OpenDatabaseSnafu { path: path.clone() })?;
        debug!(path = %path.display(), "High-score database opened");
        Ok(Self { db, path })
    }
    /// Save the score in the DB and then shrink the DB to `MAX_SCORE_ENTRIES`
    /// if the score is not in the best `MAX_SCORE_ENTRIES`, it will not be inserted
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn save_score(&self, score: &mut HighScore) -> Result<Option<usize>> {
        debug!(path = %self.path.display(), score = score.score, "Saving high score");
        let encoded = toml::to_string(&score).context(EncodeSnafu {
            score: score.score,
            date: score.date,
        })?;
        let rank = self.get_rank(score.score)?;
        //If the score to save is among the top ranks, we save
        if rank.is_some() {
            // 1. Calculate the score index key prefix (for sorting, so Big Endian)
            let score_key = (u32::MAX - score.score).to_be_bytes();

            // 2. Get a globally unique, monotonically increasing ID from sled (u64).
            // Keep it on the pending score so a retry cannot create a duplicate.
            let unique_id = if let Some(id) = score.database_id {
                id
            } else {
                let id = self.db.generate_id().with_context(|_| GenerateIdSnafu {
                    path: self.path.clone(),
                })?;
                score.database_id = Some(id);
                id
            };
            let unique_id_bytes = unique_id.to_be_bytes();

            let mut final_key = score_key.to_vec();
            final_key.extend_from_slice(&unique_id_bytes);

            self.db
                .insert(&final_key, encoded.as_bytes())
                .with_context(|_| SaveScoreSnafu {
                    path: self.path.clone(),
                    key: final_key,
                    score: score.score,
                })?;

            //Now Shrink DB
            self.shrink_db()?;
            info!(path = %self.path.display(), score = score.score, ?rank, "High score saved");
        }
        Ok(rank)
    }
    /// Shrink the DB to `MAX_SCORE_ENTRIES` size
    ///
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn shrink_db(&self) -> Result<()> {
        debug!(path = %self.path.display(), limit = MAX_SCORE_ENTRIES, "Trimming high scores");
        let mut removed = 0;
        let mut batch = sled::Batch::default();
        for (index, item) in self.db.iter().enumerate() {
            let (key, _) = item.with_context(|_| ReadEntrySnafu {
                path: self.path.clone(),
                index,
            })?;
            if index >= MAX_SCORE_ENTRIES {
                batch.remove(key);
                removed += 1;
            }
        }
        self.db
            .apply_batch(batch)
            .with_context(|_| TrimDatabaseSnafu {
                path: self.path.clone(),
                limit: MAX_SCORE_ENTRIES,
            })?;
        debug!(path = %self.path.display(), removed, "High scores trimmed; flushing database");
        self.db.flush().with_context(|_| FlushDatabaseSnafu {
            path: self.path.clone(),
        })?;
        debug!(path = %self.path.display(), "High-score database flushed");
        Ok(())
    }

    /// # Errors
    /// Returns database read errors or invalid stored score data.
    pub fn get_top_scores(&self) -> Result<Vec<HighScore>> {
        debug!(path = %self.path.display(), "Loading top scores");
        let scores = self
            .db
            .iter()
            .take(MAX_SCORE_ENTRIES)
            .enumerate()
            .map(|(index, item)| {
                let (key, value) = item.with_context(|_| ReadEntrySnafu {
                    path: self.path.clone(),
                    index,
                })?;
                let text = std::str::from_utf8(&value).with_context(|_| Utf8Snafu {
                    path: self.path.clone(),
                    key: key.to_vec(),
                    data: value.to_vec(),
                })?;
                toml::from_str(text).with_context(|_| DecodeSnafu {
                    path: self.path.clone(),
                    key: key.to_vec(),
                    data: text.to_owned(),
                })
            })
            .collect::<Result<Vec<HighScore>>>()?;
        debug!(path = %self.path.display(), count = scores.len(), "Top scores loaded");
        Ok(scores)
    }
    /// Get the rank of the score submitted among the `MAX_SCORE_ENTRIES`
    /// Use the Sled lexicographic order to have it free
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn get_rank(&self, player_score_value: u32) -> Result<Option<usize>> {
        let mut rank = 1;
        let logic = |rank: &usize| {
            //Check we save only up to MAX_SCORE_ENTRIES score
            if *rank <= MAX_SCORE_ENTRIES {
                info!(
                    Ranked = rank,
                    PlayerScore = player_score_value,
                    "Player score is in the top 10 scores ! "
                );
                return Ok(Some(*rank));
            }
            info!(
                PlayerScore = player_score_value,
                "Player score is not in the top 10 scores ! "
            );
            Ok(None)
        };
        // We iterate on our sled DB, which is lexicography sorted, so we iterated by top score
        // to bottom score :)
        for (index, item) in self.db.iter().enumerate() {
            let (key, _value) = item.with_context(|_| ReadEntrySnafu {
                path: self.path.clone(),
                index,
            })?;
            // Key 4 first bytes is the score saved as (u32::MAX - score)
            ensure!(
                key.len() >= 4,
                InvalidKeySnafu {
                    path: self.path.clone(),
                    key: key.to_vec()
                }
            );
            let current_key_score_bytes = [key[0], key[1], key[2], key[3]];
            let current_key_score = u32::MAX - u32::from_be_bytes(current_key_score_bytes);
            debug!(current_key_score, "Current key score with rank:{}", rank);
            // We found the ranking! (as we compare from top score to bottom)
            if player_score_value >= current_key_score {
                return logic(&rank);
            }
            rank += 1;
        }
        //In case we do not have all entry fulfilled, any score will be saved, even if superior to none
        logic(&rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn retry_after_partial_save_does_not_duplicate_score() {
        let dir = tempdir().unwrap();
        let mut score = HighScore::new("S".into(), 100, "Normal".into(), 1);
        {
            let manager = HighScoreManager::new_with_custom_path(dir.path()).unwrap();
            // Simulate an insert that succeeded before a later operation failed.
            let mut key = (u32::MAX - score.score).to_be_bytes().to_vec();
            let id = manager.db.generate_id().unwrap();
            score.database_id = Some(id);
            key.extend_from_slice(&id.to_be_bytes());
            manager
                .db
                .insert(key, toml::to_string(&score).unwrap().as_bytes())
                .unwrap();
            assert_eq!(manager.save_score(&mut score).unwrap(), Some(1));
            assert_eq!(manager.save_score(&mut score).unwrap(), Some(1));
        }
        let manager = HighScoreManager::new_with_custom_path(dir.path()).unwrap();
        assert_eq!(manager.get_top_scores().unwrap().len(), 1);
    }

    #[test]
    fn distinct_scores_use_monotonic_ids_even_with_identical_timestamps() {
        let dir = tempdir().unwrap();
        let manager = HighScoreManager::new_with_custom_path(dir.path()).unwrap();
        let mut first = HighScore::new("S".into(), 100, "Normal".into(), 1);
        let mut second = first.clone();
        manager.save_score(&mut first).unwrap();
        manager.save_score(&mut second).unwrap();
        assert!(first.database_id.unwrap() < second.database_id.unwrap());
        assert_eq!(manager.get_top_scores().unwrap().len(), 2);
    }

    #[test]
    fn invalid_data_returns_typed_errors() {
        let dir = tempdir().unwrap();
        let manager = HighScoreManager::new_with_custom_path(dir.path()).unwrap();
        manager.db.insert(b"x", &[255]).unwrap();
        assert!(matches!(
            manager.get_rank(100),
            Err(HighScoreError::InvalidKey { path, key, .. }) if path == dir.path() && key == b"x"
        ));
        assert!(matches!(
            manager.get_top_scores(),
            Err(HighScoreError::Utf8 { path, key, data, .. })
                if path == dir.path() && key == b"x" && data == [255]
        ));
        manager.db.insert(b"x", b"invalid toml").unwrap();
        assert!(matches!(
            manager.get_top_scores(),
            Err(HighScoreError::Decode { path, key, data, .. })
                if path == dir.path() && key == b"x" && data == "invalid toml"
        ));
    }

    #[test]
    fn report_includes_context_and_underlying_cause() {
        let dir = tempdir().unwrap();
        let manager = HighScoreManager::new_with_custom_path(dir.path()).unwrap();
        manager.db.insert(b"x", &[255]).unwrap();
        let error = manager.get_top_scores().unwrap_err();
        let cause = std::error::Error::source(&error).unwrap().to_string();
        let report = crate::game_logic::logger::error_formatting::format_error_chain(&error);
        let HighScoreError::Utf8 { location, .. } = &error else {
            panic!("expected UTF-8 error");
        };
        assert_eq!(location.file(), file!());
        assert!(!error.to_string().contains(&location.to_string()));
        assert!(
            report
                .lines()
                .next()
                .unwrap()
                .contains(&location.to_string())
        );
        assert_eq!(report.lines().count(), 2);
        assert!(report.contains(&dir.path().display().to_string()));
        assert!(report.contains("[78]"));
        assert!(report.contains("[ff]"));
        assert!(report.contains(&cause));
    }

    #[test]
    fn open_can_be_retried_after_fixing_the_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("scores");
        std::fs::write(&path, "not a directory").unwrap();
        assert!(matches!(
            HighScoreManager::new_with_custom_path(&path),
            Err(HighScoreError::OpenDatabase { path: error_path, .. }) if error_path == path
        ));
        std::fs::remove_file(&path).unwrap();
        let manager = HighScoreManager::new_with_custom_path(&path).unwrap();
        manager
            .save_score(&mut HighScore::new("S".into(), 100, "Normal".into(), 1))
            .unwrap();
        assert_eq!(manager.get_top_scores().unwrap().len(), 1);
    }

    #[test]
    fn test_high_score_save_and_load() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_high_scores.db");
        let manager = HighScoreManager::new_with_custom_path(db_path).unwrap();

        let mut score = HighScore::new("🐍•".to_string(), 100, "Normal".into(), 2);

        manager.save_score(&mut score).unwrap();

        let top_scores = manager.get_top_scores().unwrap();
        assert_eq!(top_scores.len(), 1);
        assert_eq!(top_scores[0].score, 100);
        assert_eq!(top_scores[0].symbols, "🐍•");
        assert_eq!(top_scores[0].snake_growth_factor, 2);
    }

    #[test]
    fn test_old_high_score_defaults_growth_factor() {
        let score = HighScore::new("🐍•".to_string(), 100, "Normal".into(), 2);
        let legacy_toml = toml::to_string(&score)
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with("snake_growth_factor"))
            .collect::<Vec<_>>()
            .join("\n");

        let loaded: HighScore = toml::from_str(&legacy_toml).unwrap();
        assert_eq!(loaded.snake_growth_factor, 1);
    }

    #[test]
    fn test_ranking() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_rank.db");
        let manager = HighScoreManager::new_with_custom_path(db_path).unwrap();

        let scores = vec![200, 50, 100];
        for s in scores {
            manager
                .save_score(&mut HighScore::new(
                    "S".to_string(),
                    s,
                    "Normal 🐢".into(),
                    1,
                ))
                .unwrap();
        }

        assert_eq!(manager.get_rank(250).unwrap(), Some(1));
        assert_eq!(manager.get_rank(200).unwrap(), Some(1));
        assert_eq!(manager.get_rank(150).unwrap(), Some(2));
        assert_eq!(manager.get_rank(100).unwrap(), Some(2));
        assert_eq!(manager.get_rank(75).unwrap(), Some(3));
        assert_eq!(manager.get_rank(50).unwrap(), Some(3));
        assert_eq!(manager.get_rank(10).unwrap(), Some(4));
    }
}

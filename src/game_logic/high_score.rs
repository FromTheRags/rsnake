use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use tracing::{debug, info};

const MAX_SCORE_ENTRIES: usize = 10;
const DB_FILE: &str = "high_scores.db";
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HighScore {
    pub symbols: String,
    pub score: u32,
    pub speed: String,
    pub date: DateTime<Utc>,
    pub version: String,
}
impl HighScore {
    #[must_use]
    pub fn new(symbols: String, score: u32, speed: String) -> Self {
        HighScore {
            symbols,
            score,
            speed,
            date: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
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
/// - Already used for arguments serialisation, avoid a new dependency
///   (and the bincode crate story is a lesson-teller)
/// - NB: on other projects with others constrains postcard,rkyv,or borsh
pub struct HighScoreManager {
    db: Db,
}

impl HighScoreManager {
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn new() -> Result<Self, sled::Error> {
        let manager = HighScoreManager::new_with_custom_path(DB_FILE)?;
        Ok(manager)
    }
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn new_with_custom_path<P: AsRef<Path>>(path: P) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
    /// Save the score in the DB and then shrink the DB to `MAX_SCORE_ENTRIES`
    /// if the score is not in the best `MAX_SCORE_ENTRIES`, it will not be inserted
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn save_score(
        &self,
        score: &HighScore,
    ) -> Result<Option<usize>, Box<dyn std::error::Error>> {
        let encoded = toml::to_string(&score)?;
        let rank = self.get_rank(score.score)?;
        //If the score to save is among the top ranks, we save
        if rank.is_some() {
            // 1. Calculate the score index key prefix (for sorting, so Big Endian)
            let score_key = (u32::MAX - score.score).to_be_bytes();

            // 2. Get a globally unique, monotonically increasing ID from sled (u64)
            // This replaces the timestamp for uniqueness.
            let unique_id = self.db.generate_id()?;
            let unique_id_bytes = unique_id.to_be_bytes();

            // 3. Combine them to form the final key
            //NB: if not uniq will overwrite the previous value for this key as sled is like a hashMap<[u8],[u8]>
            //And NOT a HashMap<[u8],Vec<[u8]>>
            let mut final_key = score_key.to_vec();
            final_key.extend_from_slice(&unique_id_bytes);

            self.db.insert(final_key, encoded.as_bytes())?;
            self.db.flush()?;

            //Now Shrink DB
            self.shrink_db()?;
        }
        Ok(rank)
    }
    /// Shrink the DB to `MAX_SCORE_ENTRIES` size
    ///
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn shrink_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut iter = self.db.iter();

        // The iterator returns Result <(IVec, IVec)>, so we need to chain unwrap/error checks
        // to get the key of the element to start the deletion range at.
        let key_to_start_deleting_from = iter
            .nth(MAX_SCORE_ENTRIES)
            .map_or(Ok(None), |res| res.map(|(k, _)| Some(k)))?;

        if let Some(start_key) = key_to_start_deleting_from {
            // We iterate from 'start_key' up to the unbounded end of the tree.
            let keys_to_delete = self.db.range(start_key..);

            // 2.3. Build the deletion batch.
            let mut batch = sled::Batch::default();

            for key_value_result in keys_to_delete {
                let (key, _) = key_value_result?;
                batch.remove(key);
            }

            // 2.4. Apply the batch.
            self.db.apply_batch(batch)?;
        }
        self.db.flush()?;
        // If .skip(keep_count).next() returned None, there are 10 or fewer entries, so do nothing.
        Ok(())
    }

    #[must_use]
    pub fn get_top_scores(&self) -> Vec<HighScore> {
        let mut high_score_entries: Vec<HighScore> = Vec::new();
        // We iterate on our sled DB, which is lexicography sorted, so we iterated by top score
        // to bottom score :)
        for item in self.db.iter() {
            //For each item we gonna get utf8 value
            let (_key, value) = item.expect("error in db while iterating to get score");
            if let Ok(s_toml) = std::str::from_utf8(&value) {
                //we get the toml back!
                //Toml crate uses serde under the hood, so conversion to
                // Highscore is possible thanks to Serde Deserialization macro applied on Highscore struct
                // So we get a Highscore struct
                if let Ok(high_score_entry) = toml::from_str(s_toml) {
                    //We add it to our vec
                    high_score_entries.push(high_score_entry);
                    //get only the best score, limited to <limit> usually 10,
                    // Normally not need because of shrinking but in case of
                    if high_score_entries.len() >= MAX_SCORE_ENTRIES {
                        break;
                    }
                }
            }
        }
        high_score_entries
    }
    /// Get the rank of the score submitted among the `MAX_SCORE_ENTRIES`
    /// Use the Sled lexicographic order to have it free
    /// # Errors
    ///
    /// If DB reads / writes issues
    pub fn get_rank(
        &self,
        player_score_value: u32,
    ) -> Result<Option<usize>, Box<dyn std::error::Error>> {
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
        for item in self.db.iter() {
            let (key, _value) = item?;
            // Key 4 first bytes is the score saved as (u32::MAX - score)
            let current_key_score_bytes: [u8; 4] = key[0..4].try_into()?;
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
    fn test_high_score_save_and_load() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_high_scores.db");
        let manager = HighScoreManager::new_with_custom_path(db_path).unwrap();

        let score = HighScore::new("🐍•".to_string(), 100, "Normal".into());

        manager.save_score(&score).unwrap();

        let top_scores = manager.get_top_scores();
        assert_eq!(top_scores.len(), 1);
        assert_eq!(top_scores[0].score, 100);
        assert_eq!(top_scores[0].symbols, "🐍•");
    }

    #[test]
    fn test_ranking() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_rank.db");
        let manager = HighScoreManager::new_with_custom_path(db_path).unwrap();

        let scores = vec![200, 50, 100];
        for s in scores {
            manager
                .save_score(&HighScore::new("S".to_string(), s, "Normal 🐢".into()))
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

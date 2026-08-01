use crate::helpers::utils::{PasswordEntry, UserData};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// SQLCipher-backed database. The whole file is encrypted with a 32-byte
/// raw key supplied at open time (see `PRAGMA key = "x'<hex>'"`).
#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

fn to_key_pragma(key: &[u8; 32]) -> String {
    let mut hex = String::with_capacity(64);
    for byte in key {
        hex.push_str(&format!("{:02x}", byte));
    }
    hex
}

fn to_err(prefix: &str, e: impl std::fmt::Display) -> String {
    format!("{}: {}", prefix, e)
}

impl Database {
    /// Opens (creating if needed) the database and unlocks it with the given
    /// key. Fails when the key does not match an existing database.
    pub fn open(path: &Path, key: &[u8; 32]) -> Result<Self, String> {
        let conn =
            Connection::open(path).map_err(|e| to_err("Could not open the database", e))?;
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", to_key_pragma(key)))
            .map_err(|e| to_err("Failed to set the database key", e))?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Removes any pre-existing (stale) database file and creates a fresh one.
    /// Used on first registration and migration.
    pub fn open_fresh(path: &Path, key: &[u8; 32]) -> Result<Self, String> {
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| to_err("Failed to remove the old database", e))?;
        }
        Self::open(path, key)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS users (
                    u TEXT PRIMARY KEY,
                    p_h TEXT NOT NULL,
                    salt TEXT NOT NULL,
                    key_salt TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS passwords (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    u TEXT NOT NULL,
                    e_c TEXT NOT NULL,
                    nonce TEXT NOT NULL
                );",
            )
            .map_err(|e| to_err("Failed to initialize the schema", e))
    }

    pub fn get_user(&self) -> Result<Option<UserData>, String> {
        self.conn
            .query_row(
                "SELECT u, p_h, salt, key_salt FROM users LIMIT 1",
                [],
                |row| {
                    Ok(UserData {
                        u: row.get(0)?,
                        p_h: row.get(1)?,
                        salt: row.get(2)?,
                        key_salt: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| to_err("Failed to read the user", e))
    }

    pub fn insert_user(&self, user: &UserData) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO users (u, p_h, salt, key_salt) VALUES (?1, ?2, ?3, ?4)",
                params![user.u, user.p_h, user.salt, user.key_salt],
            )
            .map_err(|e| to_err("Failed to save the user", e))?;
        Ok(())
    }

    pub fn list_passwords(&self) -> Result<Vec<PasswordEntry>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, u, e_c, nonce FROM passwords ORDER BY id")
            .map_err(|e| to_err("Failed to prepare the query", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PasswordEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    u: row.get(2)?,
                    e_c: row.get(3)?,
                    nonce: row.get(4)?,
                })
            })
            .map_err(|e| to_err("Failed to read the passwords", e))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| to_err("Failed to read the passwords", e))?);
        }
        Ok(entries)
    }

    pub fn insert_password(&self, entry: &PasswordEntry) -> Result<i64, String> {
        self.conn
            .execute(
                "INSERT INTO passwords (name, u, e_c, nonce) VALUES (?1, ?2, ?3, ?4)",
                params![entry.name, entry.u, entry.e_c, entry.nonce],
            )
            .map_err(|e| to_err("Failed to add the password", e))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_password(&self, entry: &PasswordEntry) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE passwords SET name = ?1, u = ?2, e_c = ?3, nonce = ?4 WHERE id = ?5",
                params![entry.name, entry.u, entry.e_c, entry.nonce, entry.id],
            )
            .map_err(|e| to_err("Failed to update the password", e))?;
        Ok(())
    }

    pub fn delete_password(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM passwords WHERE id = ?1", params![id])
            .map_err(|e| to_err("Failed to delete the password", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("pm_db_{}.db", nanos))
    }

    #[test]
    fn crud_roundtrip() {
        let path = temp_path();
        let key = [7u8; 32];
        let db = Database::open(&path, &key).unwrap();

        assert!(db.get_user().unwrap().is_none());
        let user = UserData {
            u: "luca".into(),
            p_h: "hash".into(),
            salt: "s".into(),
            key_salt: "ks".into(),
        };
        db.insert_user(&user).unwrap();
        assert_eq!(db.get_user().unwrap().unwrap().u, "luca");

        let mut entry = PasswordEntry {
            id: 0,
            name: "Gmail".into(),
            u: "a@b.c".into(),
            e_c: "ct".into(),
            nonce: "n".into(),
        };
        let id = db.insert_password(&entry).unwrap();
        entry.id = id;
        let second = PasswordEntry {
            id: 0,
            name: "GitHub".into(),
            u: "luca".into(),
            e_c: "ct2".into(),
            nonce: "n2".into(),
        };
        db.insert_password(&second).unwrap();
        assert_eq!(db.list_passwords().unwrap().len(), 2);

        entry.name = "Gmail2".into();
        db.update_password(&entry).unwrap();
        assert_eq!(db.list_passwords().unwrap()[0].name, "Gmail2");

        db.delete_password(entry.id).unwrap();
        assert_eq!(db.list_passwords().unwrap().len(), 1);

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn wrong_key_fails() {
        let path = temp_path();
        Database::open(&path, &[1u8; 32]).unwrap();
        let err = Database::open(&path, &[2u8; 32]).unwrap_err();
        assert!(!err.is_empty());
        std::fs::remove_file(&path).unwrap();
    }
}

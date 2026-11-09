use crate::error::StorageError;
use crate::migrations::MigrationRunner;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tracing::info;

pub struct DatabaseConfig {
    pub path: Option<PathBuf>, // None = in-memory database
    pub busy_timeout_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: None,
            busy_timeout_ms: 5000,
        }
    }
}

pub struct DatabaseManager {
    config: DatabaseConfig,
    memory_uri: Option<String>,
    // Dedicated mutex-guarded write connection ensuring single-writer serialization
    write_conn: Mutex<Connection>,
}

impl DatabaseManager {
    pub fn new(config: DatabaseConfig) -> Result<Self, StorageError> {
        let (mut conn, memory_uri) = match &config.path {
            Some(path) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                (Connection::open(path)?, None)
            }
            None => {
                let uri = format!(
                    "file:mem_{}?mode=memory&cache=shared",
                    uuid::Uuid::new_v4().simple()
                );
                let c = Connection::open_with_flags(
                    &uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_URI
                        | rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                        | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
                )?;
                (c, Some(uri))
            }
        };

        Self::configure_pragmas(&mut conn, config.busy_timeout_ms)?;
        MigrationRunner::apply_all(&mut conn)?;

        info!("Database initialized successfully (FTS5 & WAL active)");

        Ok(Self {
            config,
            memory_uri,
            write_conn: Mutex::new(conn),
        })
    }

    pub fn new_in_memory() -> Result<Self, StorageError> {
        Self::new(DatabaseConfig::default())
    }

    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Self::new(DatabaseConfig {
            path: Some(path.as_ref().to_path_buf()),
            busy_timeout_ms: 5000,
        })
    }

    fn configure_pragmas(conn: &mut Connection, busy_timeout: u64) -> Result<(), StorageError> {
        conn.busy_timeout(std::time::Duration::from_millis(busy_timeout))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(())
    }

    /// Acquires the dedicated single-writer lock and returns the guarded connection.
    pub fn get_writer(&self) -> Result<MutexGuard<'_, Connection>, StorageError> {
        self.write_conn
            .lock()
            .map_err(|_| StorageError::LockTimeout)
    }

    /// Executes a write operation within the single-writer lock.
    pub fn with_writer<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&mut Connection) -> Result<R, StorageError>,
    {
        let mut conn = self
            .write_conn
            .lock()
            .map_err(|_| StorageError::LockTimeout)?;
        f(&mut conn)
    }

    /// Creates a read-only connection for concurrent non-blocking queries.
    pub fn create_reader(&self) -> Result<Connection, StorageError> {
        let conn = match &self.config.path {
            Some(path) => Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                    | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?,
            None => {
                let uri = self
                    .memory_uri
                    .as_deref()
                    .unwrap_or("file::memory:?cache=shared");
                Connection::open_with_flags(
                    uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_URI
                        | rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
                )?
            }
        };
        conn.busy_timeout(std::time::Duration::from_millis(
            self.config.busy_timeout_ms,
        ))?;
        Ok(conn)
    }

    /// Executes a read operation on a dedicated read connection.
    pub fn with_reader<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&Connection) -> Result<R, StorageError>,
    {
        let conn = self.create_reader()?;
        f(&conn)
    }
}

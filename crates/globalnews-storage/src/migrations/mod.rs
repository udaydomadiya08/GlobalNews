use crate::error::StorageError;
use rusqlite::Connection;
use tracing::info;

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "001_initial_schema",
    sql: include_str!("001_initial_schema.sql"),
}];

pub struct MigrationRunner;

impl MigrationRunner {
    pub fn apply_all(conn: &mut Connection) -> Result<usize, StorageError> {
        // Ensure schema_migrations table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
            [],
        )?;

        let mut applied_count = 0;

        for migration in MIGRATIONS {
            let already_applied: bool = conn
                .query_row(
                    "SELECT 1 FROM schema_migrations WHERE version = ?1",
                    [migration.version],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if !already_applied {
                info!(
                    "Applying migration {} ({})...",
                    migration.version, migration.name
                );
                let tx = conn.transaction()?;
                tx.execute_batch(migration.sql)?;
                tx.execute(
                    "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        migration.version,
                        migration.name,
                        chrono::Utc::now().to_rfc3339()
                    ],
                )?;
                tx.commit()?;
                applied_count += 1;
            }
        }

        Ok(applied_count)
    }

    pub fn get_current_version(conn: &Connection) -> Result<i32, StorageError> {
        let version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(version)
    }
}

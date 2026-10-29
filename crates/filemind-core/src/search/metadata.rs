use rusqlite::{Connection, Result};

use crate::db::repository::Database;
use crate::models::{FileListQuery, FileListResponse};

pub struct MetadataSearchEngine;

impl MetadataSearchEngine {
    /// Execute high-speed indexed search over file records.
    pub fn search(conn: &Connection, query: &FileListQuery) -> Result<FileListResponse> {
        Database::query_files(conn, query)
    }
}

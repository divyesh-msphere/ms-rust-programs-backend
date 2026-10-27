use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_event;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::document::{
    Document, DocumentFilter, DocumentRepository, DocumentSummary, Folder, GENERAL_FOLDER, Location, MODULE_KEY, NEEDS_REVIEW,
    NewDocument, NewFolder, OK, PENDING_SCAN, QUARANTINED, RECENT_LIMIT, ReviewDecision, ScanResult, UPLOADING,
    UpdateDocument, UpdateFolder, is_counted, reviewed_status,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{FromRow, PgConnection, Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

const DOC_COLS: &str = "id, program_id, folder_id, name, content_type, size_bytes, file_key, status, source_type, source_id, uploaded_by, \
    checksum, scan_time, review_note, reviewed_by, reviewed_time, create_time, update_time";
const FOLDER_COLS: &str = "id, program_id, parent_id, name, key, system, document_count, create_time, update_time";

#[derive(Debug, FromRow)]
struct DocRow {
    id: Uuid,
    program_id: Uuid,
    folder_id: Option<Uuid>,
    name: String,
    content_type: String,
    size_bytes: i64,
    file_key: String,
    status: String,
    source_type: Option<String>,
    source_id: Option<String>,
    uploaded_by: Option<String>,
    checksum: Option<String>,
    scan_time: Option<DateTime<Utc>>,
    review_note: Option<String>,
    reviewed_by: Option<String>,
    reviewed_time: Option<DateTime<Utc>>,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

impl From<DocRow> for Document {
    fn from(r: DocRow) -> Self {
        Self {
            id: r.id,
            program_id: r.program_id,
            folder_id: r.folder_id,
            name: r.name,
            content_type: r.content_type,
            size_bytes: r.size_bytes,
            file_key: r.file_key,
            status: r.status,
            source_type: r.source_type,
            source_id: r.source_id,
            uploaded_by: r.uploaded_by,
            checksum: r.checksum,
            scan_time: r.scan_time,
            review_note: r.review_note,
            reviewed_by: r.reviewed_by,
            reviewed_time: r.reviewed_time,
            create_time: r.create_time,
            update_time: r.update_time,
        }
    }
}

#[derive(Debug, FromRow)]
struct FolderRow {
    id: Uuid,
    program_id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    key: Option<String>,
    system: bool,
    document_count: i32,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

impl From<FolderRow> for Folder {
    fn from(r: FolderRow) -> Self {
        Self {
            id: r.id,
            program_id: r.program_id,
            parent_id: r.parent_id,
            name: r.name,
            key: r.key,
            system: r.system,
            document_count: r.document_count,
            create_time: r.create_time,
            update_time: r.update_time,
        }
    }
}

pub struct DocumentRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl DocumentRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

/// The program must have the Documents module switched on.
async fn ensure_enabled(conn: &mut PgConnection, program_id: Uuid) -> Result<(), ApplicationError> {
    let on: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM program_modules pm JOIN module_definitions md ON md.id = pm.module_id \
                        WHERE pm.program_id = $1 AND md.key = $2 AND pm.enabled)",
    )
    .bind(program_id)
    .bind(MODULE_KEY)
    .fetch_one(conn)
    .await
    .map_err(db_error)?;
    if on {
        Ok(())
    } else {
        Err(ApplicationError::failed_precondition(
            "this program does not use Documents: switch the \"Documents Panel\" module on first",
        ))
    }
}

/// The page's totals, in the same transaction as the change, so the page never runs `COUNT(*)`.
async fn adjust_storage(
    conn: &mut PgConnection,
    program_id: Uuid,
    files: i32,
    bytes: i64,
    needs_review: i32,
    quarantined: i32,
) -> Result<(), ApplicationError> {
    sqlx::query(
        "INSERT INTO program_storage (program_id, file_count, total_bytes, needs_review_count, quarantined_count) \
         VALUES ($1, GREATEST($2, 0), GREATEST($3, 0), GREATEST($4, 0), GREATEST($5, 0)) \
         ON CONFLICT (program_id) DO UPDATE SET \
            file_count = GREATEST(program_storage.file_count + $2, 0), \
            total_bytes = GREATEST(program_storage.total_bytes + $3, 0), \
            needs_review_count = GREATEST(program_storage.needs_review_count + $4, 0), \
            quarantined_count = GREATEST(program_storage.quarantined_count + $5, 0), \
            update_time = now()",
    )
    .bind(program_id)
    .bind(files)
    .bind(bytes)
    .bind(needs_review)
    .bind(quarantined)
    .execute(conn)
    .await
    .map_err(db_error)?;
    Ok(())
}

async fn adjust_folder(conn: &mut PgConnection, folder_id: Option<Uuid>, delta: i32) -> Result<(), ApplicationError> {
    let Some(folder_id) = folder_id else { return Ok(()) };
    sqlx::query("UPDATE document_folders SET document_count = GREATEST(document_count + $2, 0), update_time = now() WHERE id = $1")
        .bind(folder_id)
        .bind(delta)
        .execute(conn)
        .await
        .map_err(db_error)?;
    Ok(())
}

/// How a status moves the "Needs review" and "Quarantined" counters.
fn status_counts(status: &str) -> (i32, i32) {
    match status {
        NEEDS_REVIEW => (1, 0),
        QUARANTINED => (0, 1),
        _ => (0, 0),
    }
}

async fn fetch_doc(conn: &mut PgConnection, program_id: Uuid, id: Uuid, lock: bool) -> Result<Document, ApplicationError> {
    let query = format!(
        "SELECT {DOC_COLS} FROM documents WHERE id = $1 AND program_id = $2 AND state = 'ACTIVE'{}",
        if lock { " FOR UPDATE" } else { "" }
    );
    sqlx::query_as::<_, DocRow>(&query)
        .bind(id)
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .map(Document::from)
        .ok_or_else(|| ApplicationError::not_found("Document", id))
}

async fn folder_exists(conn: &mut PgConnection, program_id: Uuid, id: Uuid) -> Result<(), ApplicationError> {
    let found: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM document_folders WHERE id = $1 AND program_id = $2)")
        .bind(id)
        .bind(program_id)
        .fetch_one(conn)
        .await
        .map_err(db_error)?;
    found.then_some(()).ok_or_else(|| ApplicationError::not_found("Folder", id))
}

async fn fetch_folder(conn: &mut PgConnection, program_id: Uuid, id: Uuid, lock: bool) -> Result<Folder, ApplicationError> {
    let query = format!(
        "SELECT {FOLDER_COLS} FROM document_folders WHERE id = $1 AND program_id = $2{}",
        if lock { " FOR UPDATE" } else { "" }
    );
    sqlx::query_as::<_, FolderRow>(&query)
        .bind(id)
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .map(Folder::from)
        .ok_or_else(|| ApplicationError::not_found("Folder", id))
}

fn folder_name_taken(err: sqlx::Error) -> ApplicationError {
    match &err {
        sqlx::Error::Database(db) if db.constraint() == Some("uq_document_folders_name") => {
            ApplicationError::already_exists("a folder with this name already exists here")
        }
        _ => db_error(err),
    }
}

#[async_trait]
impl DocumentRepository for DocumentRepositoryPostgresImpl {
    async fn summary(&self, program_id: Uuid) -> Result<DocumentSummary, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        ensure_enabled(&mut conn, program_id).await?;
        let counters: Option<(i32, i64, i32, i32)> = sqlx::query_as(
            "SELECT file_count, total_bytes, needs_review_count, quarantined_count FROM program_storage WHERE program_id = $1",
        )
        .bind(program_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?;
        let (files, bytes, needs_review, quarantined) = counters.unwrap_or((0, 0, 0, 0));
        let folders = list_folder_rows(&mut conn, program_id).await?;
        Ok(DocumentSummary { files, bytes, needs_review, quarantined, folders })
    }

    async fn list_folders(&self, program_id: Uuid) -> Result<Vec<Folder>, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        ensure_enabled(&mut conn, program_id).await?;
        list_folder_rows(&mut conn, program_id).await
    }

    async fn create_folder(&self, input: NewFolder) -> Result<Folder, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, input.program_id).await?;
        if let Some(parent) = input.parent_id {
            folder_exists(&mut tx, input.program_id, parent).await?;
        }
        let id = Uuid::now_v7();
        let query = format!(
            "INSERT INTO document_folders (id, program_id, parent_id, name) VALUES ($1, $2, $3, $4) RETURNING {FOLDER_COLS}"
        );
        let folder = Folder::from(
            sqlx::query_as::<_, FolderRow>(&query)
                .bind(id)
                .bind(input.program_id)
                .bind(input.parent_id)
                .bind(&input.name)
                .fetch_one(&mut *tx)
                .await
                .map_err(folder_name_taken)?,
        );
        record_event(&mut tx, input.program_id, &input.actor, "created", "document_folder", &id.to_string(), None, Some(json!({ "name": folder.name, "parent_id": folder.parent_id })))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(folder)
    }

    async fn update_folder(&self, program_id: Uuid, id: Uuid, input: UpdateFolder) -> Result<Folder, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, program_id).await?;
        let existing = fetch_folder(&mut tx, program_id, id, true).await?;
        let Some(name) = input.name.as_deref().filter(|n| *n != existing.name) else {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        };
        if existing.system {
            return Err(ApplicationError::conflict("a system folder cannot be renamed"));
        }
        let query = format!("UPDATE document_folders SET name = $3, update_time = now() WHERE id = $1 AND program_id = $2 RETURNING {FOLDER_COLS}");
        let updated = Folder::from(
            sqlx::query_as::<_, FolderRow>(&query)
                .bind(id)
                .bind(program_id)
                .bind(name)
                .fetch_one(&mut *tx)
                .await
                .map_err(folder_name_taken)?,
        );
        record_event(&mut tx, program_id, &input.actor, "renamed", "document_folder", &id.to_string(), Some(json!({ "name": existing.name })), Some(json!({ "name": updated.name })))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(updated)
    }

    async fn delete_folder(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, program_id).await?;
        let existing = fetch_folder(&mut tx, program_id, id, true).await?;
        if existing.system {
            return Err(ApplicationError::conflict("a system folder cannot be deleted"));
        }
        let (has_children, has_files): (bool, bool) = sqlx::query_as(
            "SELECT EXISTS (SELECT 1 FROM document_folders WHERE parent_id = $1), \
                    EXISTS (SELECT 1 FROM documents WHERE folder_id = $1 AND state = 'ACTIVE')",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        if has_children || has_files {
            return Err(ApplicationError::conflict("this folder is not empty: move or delete its files and folders first"));
        }
        sqlx::query("DELETE FROM document_folders WHERE id = $1").bind(id).execute(&mut *tx).await.map_err(db_error)?;
        record_event(&mut tx, program_id, actor, "deleted", "document_folder", &id.to_string(), Some(json!({ "name": existing.name })), None)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }

    async fn list(
        &self,
        program_id: Uuid,
        filter: &DocumentFilter,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Document>, Option<Uuid>), ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        ensure_enabled(&mut conn, program_id).await?;

        // The state and status are LITERALS so the planner can use the partial indexes (All files and Recent
        // read one backward range; Needs review and Quarantined read almost nothing).
        let mut qb = QueryBuilder::<Postgres>::new(format!("SELECT {DOC_COLS} FROM documents WHERE program_id = "));
        qb.push_bind(program_id);
        qb.push(" AND state = 'ACTIVE'");
        match filter.location {
            Some(Location::NeedsReview) => qb.push(" AND status = 'needs_review'"),
            Some(Location::Quarantined) => qb.push(" AND status = 'quarantined'"),
            _ => qb.push(" AND status <> 'uploading'"),
        };
        if let Some(folder) = filter.folder_id {
            qb.push(" AND folder_id = ").push_bind(folder);
        }
        if let Some(prefix) = &filter.name_prefix {
            let escaped = prefix.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            qb.push(" AND lower(name) LIKE ").push_bind(format!("{escaped}%"));
        }
        let recent = filter.location == Some(Location::Recent);
        if let (Some(token), false) = (page_token, recent) {
            qb.push(" AND id < ").push_bind(token);
        }
        // "Recent" is just the newest few and has no next page.
        let limit = if recent { RECENT_LIMIT } else { page_size + 1 };
        qb.push(" ORDER BY id DESC LIMIT ").push_bind(limit);

        let mut rows = qb.build_query_as::<DocRow>().fetch_all(&mut *conn).await.map_err(db_error)?;
        let next_page_token = if !recent && rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|r| r.id)
        } else {
            None
        };
        Ok((rows.into_iter().map(Document::from).collect(), next_page_token))
    }

    async fn create(&self, input: NewDocument, max_files: i64) -> Result<Document, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, input.program_id).await?;

        let files: Option<i32> = sqlx::query_scalar("SELECT file_count FROM program_storage WHERE program_id = $1")
            .bind(input.program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?;
        if i64::from(files.unwrap_or(0)) >= max_files {
            return Err(ApplicationError::failed_precondition(&format!("this program has reached its limit of {max_files} files")));
        }

        let folder_id = match input.folder_id {
            Some(folder) => {
                folder_exists(&mut tx, input.program_id, folder).await?;
                Some(folder)
            }
            None => sqlx::query_scalar("SELECT id FROM document_folders WHERE program_id = $1 AND key = $2")
                .bind(input.program_id)
                .bind(GENERAL_FOLDER)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db_error)?,
        };

        // The server picks the object key: a user-supplied name is only a label and can never choose a path.
        let id = Uuid::now_v7();
        let file_key = format!("programs/{}/documents/{id}", input.program_id);
        let query = format!(
            "INSERT INTO documents (id, program_id, folder_id, name, content_type, size_bytes, file_key, status, uploaded_by, checksum) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'uploading', $8, $9) RETURNING {DOC_COLS}"
        );
        let created = Document::from(
            sqlx::query_as::<_, DocRow>(&query)
                .bind(id)
                .bind(input.program_id)
                .bind(folder_id)
                .bind(&input.name)
                .bind(input.content_type.trim().to_lowercase())
                .bind(input.size_bytes)
                .bind(&file_key)
                .bind(&input.uploaded_by)
                .bind(input.checksum.as_deref().map(str::to_lowercase))
                .fetch_one(&mut *tx)
                .await
                .map_err(db_error)?,
        );
        // Nothing is counted and nothing is audited yet: the file is hidden until its upload completes.
        tx.commit().await.map_err(db_error)?;
        Ok(created)
    }

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Document, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        ensure_enabled(&mut conn, program_id).await?;
        fetch_doc(&mut conn, program_id, id, false).await
    }

    async fn complete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<Document, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, program_id).await?;
        let existing = fetch_doc(&mut tx, program_id, id, true).await?;
        if existing.status != UPLOADING {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        }
        let query = format!("UPDATE documents SET status = 'pending_scan', update_time = now() WHERE id = $1 RETURNING {DOC_COLS}");
        let done = Document::from(sqlx::query_as::<_, DocRow>(&query).bind(id).fetch_one(&mut *tx).await.map_err(db_error)?);

        adjust_storage(&mut tx, program_id, 1, done.size_bytes, 0, 0).await?;
        adjust_folder(&mut tx, done.folder_id, 1).await?;
        // The outbox event is what the scanner listens to.
        record_event(
            &mut tx,
            program_id,
            actor,
            "uploaded",
            "document",
            &id.to_string(),
            None,
            Some(json!({ "document_id": id, "name": done.name, "content_type": done.content_type, "size_bytes": done.size_bytes, "file_key": done.file_key, "status": PENDING_SCAN })),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(done)
    }

    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateDocument) -> Result<Document, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, program_id).await?;
        let existing = fetch_doc(&mut tx, program_id, id, true).await?;

        let name = input.name.clone().unwrap_or_else(|| existing.name.clone());
        let folder_id = match input.folder_id {
            Some(folder) if Some(folder) != existing.folder_id => {
                folder_exists(&mut tx, program_id, folder).await?;
                Some(folder)
            }
            _ => existing.folder_id,
        };
        if name == existing.name && folder_id == existing.folder_id {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        }

        let query = format!("UPDATE documents SET name = $2, folder_id = $3, update_time = now() WHERE id = $1 RETURNING {DOC_COLS}");
        let updated = Document::from(sqlx::query_as::<_, DocRow>(&query).bind(id).bind(&name).bind(folder_id).fetch_one(&mut *tx).await.map_err(db_error)?);

        if is_counted(&existing.status) && folder_id != existing.folder_id {
            adjust_folder(&mut tx, existing.folder_id, -1).await?;
            adjust_folder(&mut tx, folder_id, 1).await?;
        }
        record_event(
            &mut tx,
            program_id,
            &input.actor,
            "updated",
            "document",
            &id.to_string(),
            Some(json!({ "name": existing.name, "folder_id": existing.folder_id })),
            Some(json!({ "name": updated.name, "folder_id": updated.folder_id })),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(updated)
    }

    async fn delete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, program_id).await?;
        let existing = fetch_doc(&mut tx, program_id, id, true).await?;
        sqlx::query("UPDATE documents SET state = 'DELETED', update_time = now() WHERE id = $1").bind(id).execute(&mut *tx).await.map_err(db_error)?;

        if is_counted(&existing.status) {
            let (review, quarantined) = status_counts(&existing.status);
            adjust_storage(&mut tx, program_id, -1, -existing.size_bytes, -review, -quarantined).await?;
            adjust_folder(&mut tx, existing.folder_id, -1).await?;
        }
        record_event(&mut tx, program_id, actor, "deleted", "document", &id.to_string(), Some(json!({ "name": existing.name, "status": existing.status })), None)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }

    async fn review(
        &self,
        program_id: Uuid,
        id: Uuid,
        decision: ReviewDecision,
        note: Option<String>,
        actor: &str,
    ) -> Result<Document, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        ensure_enabled(&mut tx, program_id).await?;
        let existing = fetch_doc(&mut tx, program_id, id, true).await?;
        let status = reviewed_status(&existing.status, decision)?;

        let query = format!(
            "UPDATE documents SET status = $2, review_note = $3, reviewed_by = $4, reviewed_time = now(), update_time = now() \
             WHERE id = $1 RETURNING {DOC_COLS}"
        );
        let reviewed = Document::from(
            sqlx::query_as::<_, DocRow>(&query).bind(id).bind(status).bind(&note).bind(actor).fetch_one(&mut *tx).await.map_err(db_error)?,
        );
        let (review, quarantined) = status_counts(status);
        adjust_storage(&mut tx, program_id, 0, 0, review - 1, quarantined).await?;
        record_event(
            &mut tx,
            program_id,
            actor,
            "reviewed",
            "document",
            &id.to_string(),
            Some(json!({ "status": existing.status })),
            Some(json!({ "status": status, "decision": if decision == ReviewDecision::Approve { "approve" } else { "reject" }, "note": note })),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(reviewed)
    }

    async fn apply_scan(&self, id: Uuid, result: ScanResult, scanner: &str) -> Result<Document, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let query = format!("SELECT {DOC_COLS} FROM documents WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE");
        let existing = sqlx::query_as::<_, DocRow>(&query)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .map(Document::from)
            .ok_or_else(|| ApplicationError::not_found("Document", id))?;
        // A repeated callback, or one for a file that is not waiting for a scan, changes nothing.
        if existing.status != PENDING_SCAN {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        }

        let status = result.status();
        let query = format!("UPDATE documents SET status = $2, scan_time = now(), update_time = now() WHERE id = $1 RETURNING {DOC_COLS}");
        let scanned = Document::from(sqlx::query_as::<_, DocRow>(&query).bind(id).bind(status).fetch_one(&mut *tx).await.map_err(db_error)?);

        let (review, quarantined) = status_counts(status);
        if review != 0 || quarantined != 0 {
            adjust_storage(&mut tx, existing.program_id, 0, 0, review, quarantined).await?;
        }
        record_event(
            &mut tx,
            existing.program_id,
            scanner,
            "scanned",
            "document",
            &id.to_string(),
            Some(json!({ "status": PENDING_SCAN })),
            Some(json!({ "status": status, "result": match result { ScanResult::Clean => "clean", ScanResult::Suspicious => "suspicious", ScanResult::Infected => "infected" }, "ok": status == OK })),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(scanned)
    }
}

async fn list_folder_rows(conn: &mut PgConnection, program_id: Uuid) -> Result<Vec<Folder>, ApplicationError> {
    // The system folders first, then the rest by name: a handful of rows, counts already on them.
    let query = format!(
        "SELECT {FOLDER_COLS} FROM document_folders WHERE program_id = $1 \
         ORDER BY system DESC, key NULLS LAST, parent_id NULLS FIRST, lower(name), id"
    );
    let rows = sqlx::query_as::<_, FolderRow>(&query).bind(program_id).fetch_all(conn).await.map_err(db_error)?;
    Ok(rows.into_iter().map(Folder::from).collect())
}

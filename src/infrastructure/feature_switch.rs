// Switches finished modules on or off at start-up, without touching a line of their code.
//
// A parked feature's pages are moved out of `module_definitions` / `program_modules` into
// `parked_module_definitions` / `parked_program_modules`, so the module tree, every program's
// manifest and module selection stop showing them with no change to any query. Turning the
// feature on moves the rows back unchanged. The feature's own tables are never touched, so
// nothing a program already did with it is lost. Running this twice in a row changes nothing.
use crate::config::FeaturesConfig;
use ms_telemetry_rs::tracing::info;
use sqlx::PgPool;

/// The pages of Requirements and Applications & Approvals, as ltree paths (a path covers the
/// page itself and everything under it).
const ENROLLMENT_REVIEW_PAGES: &[&str] = &["enrollment.requirements", "enrollment.applications_approvals"];

/// Held while rows move, so two instances starting together do not interleave.
const LOCK_KEY: i64 = 7_261_027;

pub async fn apply(pool: &PgPool, features: &FeaturesConfig) -> Result<(), sqlx::Error> {
    sync(pool, "enrollment_review", ENROLLMENT_REVIEW_PAGES, features.enrollment_review).await
}

async fn sync(pool: &PgPool, feature: &str, pages: &[&str], enabled: bool) -> Result<(), sqlx::Error> {
    let pages: Vec<String> = pages.iter().map(|page| page.to_string()).collect();
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)").bind(LOCK_KEY).execute(&mut *tx).await?;

    let mut moved = 0;
    if enabled {
        // Parents before children (the parent foreign key), then the programs' rows.
        for kind in ["group", "leaf"] {
            moved += sqlx::query(
                "WITH moved AS (DELETE FROM parked_module_definitions \
                                WHERE kind = $2 AND ancestry <@ ANY($1::text[]::ltree[]) RETURNING *) \
                 INSERT INTO module_definitions SELECT * FROM moved ON CONFLICT DO NOTHING",
            )
            .bind(&pages)
            .bind(kind)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }
        moved += sqlx::query(
            "WITH moved AS (DELETE FROM parked_program_modules ppm USING module_definitions md \
                            WHERE md.id = ppm.module_id AND md.ancestry <@ ANY($1::text[]::ltree[]) RETURNING ppm.*) \
             INSERT INTO program_modules SELECT * FROM moved ON CONFLICT DO NOTHING",
        )
        .bind(&pages)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    } else {
        // The programs' rows first, then the pages: children before parents (the parent foreign key).
        moved += sqlx::query(
            "WITH moved AS (DELETE FROM program_modules pm USING module_definitions md \
                            WHERE md.id = pm.module_id AND md.ancestry <@ ANY($1::text[]::ltree[]) RETURNING pm.*) \
             INSERT INTO parked_program_modules SELECT * FROM moved",
        )
        .bind(&pages)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        for kind in ["leaf", "group"] {
            moved += sqlx::query(
                "WITH moved AS (DELETE FROM module_definitions \
                                WHERE kind = $2 AND ancestry <@ ANY($1::text[]::ltree[]) RETURNING *) \
                 INSERT INTO parked_module_definitions SELECT * FROM moved",
            )
            .bind(&pages)
            .bind(kind)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }
    }
    tx.commit().await?;
    info!("feature `{feature}` is {} ({moved} catalog rows moved)", if enabled { "on" } else { "off" });
    Ok(())
}

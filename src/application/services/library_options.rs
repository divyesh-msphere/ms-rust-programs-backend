// application/services/library_options.rs
//
// The two places a library-backed setting meets global-settings:
//   * on SAVE: every `library_id` must be a library that exists (checked BEFORE the database
//     transaction, so no outside call ever runs inside one);
//   * on READ: the library's options are filled into the manifest, live from the catalog's
//     cache, and a library that cannot be read degrades that one setting (`library_error`),
//     never the whole page.
use crate::application::errors::application_error::ApplicationError;
use crate::domain::library_catalog::{CatalogError, Library, LibraryCatalog};
use crate::domain::program_setting_value::missing_options;
use crate::domain::program_module::ProgramModule;
use crate::domain::program_setting::{ModuleChange, OptionsSource, SettingOption, library_ids};
use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

async fn fetch_all(
    catalog: &Arc<dyn LibraryCatalog>,
    ids: Vec<Uuid>,
) -> HashMap<Uuid, Result<Arc<Library>, CatalogError>> {
    // In parallel: a cold page costs one round trip, not one per library. Warm libraries
    // never leave memory.
    join_all(ids.into_iter().map(|id| {
        let catalog = catalog.clone();
        async move { (id, catalog.get(id).await) }
    }))
    .await
    .into_iter()
    .collect()
}

/// Rejects a save that points at a library global-settings does not have.
pub async fn validate_libraries(
    catalog: &Arc<dyn LibraryCatalog>,
    changes: &[ModuleChange],
) -> Result<(), ApplicationError> {
    let ids = library_ids(changes);
    if ids.is_empty() || !catalog.enabled() {
        return Ok(());
    }

    for (id, result) in fetch_all(catalog, ids).await {
        match result {
            Ok(_) => {}
            Err(CatalogError::NotFound) => {
                return Err(ApplicationError::unprocessable_entity(&format!(
                    "library {id} does not exist in global-settings"
                )));
            }
            Err(CatalogError::Unavailable(reason)) => {
                return Err(ApplicationError::failed_precondition(&format!(
                    "global-settings is unavailable, so library {id} cannot be checked: {reason}"
                )));
            }
        }
    }
    Ok(())
}

/// Fills each library-backed setting's `options` from its library (active options only, in
/// the library's own order). Manual options are already there from the database.
pub async fn resolve_options(catalog: &Arc<dyn LibraryCatalog>, modules: &mut [ProgramModule]) {
    let mut ids: Vec<Uuid> = modules
        .iter()
        .flat_map(|module| module.settings.iter())
        .filter(|setting| setting.options_source == Some(OptionsSource::Library))
        .filter_map(|setting| setting.library_id)
        .collect();
    ids.sort();
    ids.dedup();

    let libraries = if ids.is_empty() || !catalog.enabled() { HashMap::new() } else { fetch_all(catalog, ids).await };

    for setting in modules.iter_mut().flat_map(|module| module.settings.iter_mut()) {
        if setting.options_source != Some(OptionsSource::Library) {
            continue;
        }
        let Some(library_id) = setting.library_id else { continue };

        match libraries.get(&library_id) {
            Some(Ok(library)) => {
                setting.options = library
                    .options
                    .iter()
                    .filter(|option| option.active)
                    .map(|option| SettingOption {
                        id: option.id,
                        code: option.code.clone(),
                        label: option.name.clone(),
                        source: "library".to_string(),
                        active: true,
                    })
                    .collect();
                setting.library_error = None;
            }
            Some(Err(CatalogError::NotFound)) => setting.library_error = Some("not_found".to_string()),
            _ => setting.library_error = Some("unavailable".to_string()),
        }
    }

    // A saved choice whose option is no longer offered (deactivated, or gone from its library)
    // is flagged, so the page can warn instead of silently showing nothing selected. A library
    // that could not be read says nothing about its options, so it is not flagged.
    for setting in modules.iter_mut().flat_map(|module| module.settings.iter_mut()) {
        if setting.options_source.is_none() || setting.library_error.is_some() || setting.option_ids.is_empty() {
            continue;
        }
        let offered: Vec<Uuid> = setting.options.iter().map(|option| option.id).collect();
        setting.missing_option_ids = missing_options(&setting.option_ids, &offered);
    }
}

/// The ids of each library's ACTIVE options (what a value may pick), read through the cache in
/// parallel. A library that cannot be read is an `Err` for that library only.
pub async fn active_option_ids(
    catalog: &Arc<dyn LibraryCatalog>,
    ids: Vec<Uuid>,
) -> HashMap<Uuid, Result<HashSet<Uuid>, CatalogError>> {
    fetch_all(catalog, ids)
        .await
        .into_iter()
        .map(|(id, result)| {
            let options = result.map(|library| library.options.iter().filter(|o| o.active).map(|o| o.id).collect());
            (id, options)
        })
        .collect()
}

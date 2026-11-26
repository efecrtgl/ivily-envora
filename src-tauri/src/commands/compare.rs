use crate::activity::{self, Status};
use crate::compare::{self, Comparison, Source};
use crate::error::AppResult;
use crate::security::overrides;
use crate::state::AppState;
use serde_json::json;
use tauri::State;

fn kind_of(s: &Source) -> &'static str {
    match s {
        Source::Scope { .. } => "scope",
        Source::Effective => "effective",
        Source::Profile { .. } => "profile",
        Source::Project { .. } => "project",
        Source::Snapshot { .. } => "snapshot",
        Source::Import { .. } => "import",
        Source::Inline { .. } => "inline",
    }
}

#[tauri::command]
pub fn compare_sources(
    state: State<'_, AppState>,
    left: Source,
    right: Source,
) -> AppResult<Comparison> {
    let o = overrides::all(&state.db)?;
    let p = state.provider.as_ref();
    let resolve = |s: &Source| -> AppResult<Source> {
        Ok(match s {
            Source::Import { token } => {
                let b = state.imports.get(token)?;
                Source::Inline {
                    label: "import".into(),
                    items: crate::io::import::compare_items(&b, &o),
                }
            }
            other => other.clone(),
        })
    };
    let l = compare::load(&state.db, p, &o, &resolve(&left)?)?;
    let r = compare::load(&state.db, p, &o, &resolve(&right)?)?;
    let c = compare::compare(&l, &r);
    activity::record_quiet(
        &state.db,
        "compare",
        "compare",
        None,
        Status::Ok,
        Some(
            json!({ "left": kind_of(&left), "right": kind_of(&right), "changed": c.changed, "added": c.added, "removed": c.removed }),
        ),
    );
    Ok(c)
}

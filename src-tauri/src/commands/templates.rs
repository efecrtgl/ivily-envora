use crate::activity::{self, Status};
use crate::error::AppResult;
use crate::state::AppState;
use crate::templates::{self, model::Template, model::TemplateInput};
use tauri::State;

fn english(state: &AppState) -> bool {
    state.settings.read().general.language == "en"
}

#[tauri::command]
pub fn templates_list(state: State<'_, AppState>) -> AppResult<Vec<Template>> {
    templates::list(&state.db, english(&state))
}

#[tauri::command]
pub fn template_get(state: State<'_, AppState>, id: String) -> AppResult<Template> {
    templates::get(&state.db, &id, english(&state))
}

#[tauri::command]
pub fn template_create(state: State<'_, AppState>, input: TemplateInput) -> AppResult<String> {
    let id = templates::create(&state.db, &input)?;
    activity::record_quiet(
        &state.db,
        "template",
        "create",
        Some(input.name.trim()),
        Status::Ok,
        None,
    );
    Ok(id)
}

#[tauri::command]
pub fn template_update(
    state: State<'_, AppState>,
    id: String,
    input: TemplateInput,
) -> AppResult<()> {
    templates::update(&state.db, &id, &input)?;
    activity::record_quiet(
        &state.db,
        "template",
        "update",
        Some(input.name.trim()),
        Status::Ok,
        None,
    );
    Ok(())
}

#[tauri::command]
pub fn template_duplicate(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<String> {
    let new_id = templates::duplicate(&state.db, &id, &name, english(&state))?;
    activity::record_quiet(
        &state.db,
        "template",
        "duplicate",
        Some(name.trim()),
        Status::Ok,
        None,
    );
    Ok(new_id)
}

#[tauri::command]
pub fn template_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    templates::delete(&state.db, &id)?;
    activity::record_quiet(&state.db, "template", "delete", Some(&id), Status::Ok, None);
    Ok(())
}

mod commands;
mod db;
mod models;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            db::initialize_database(app.handle())
                .map_err(std::io::Error::other)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::course::list_courses,
            commands::course::get_course,
            commands::course::create_course,
            commands::course::update_course,
            commands::course::delete_course,
            commands::knowledge_point::list_knowledge_points,
            commands::knowledge_point::list_knowledge_points_by_course,
            commands::knowledge_point::get_knowledge_point_tree,
            commands::knowledge_point::get_knowledge_point_detail,
            commands::knowledge_point::create_knowledge_point,
            commands::knowledge_point::update_knowledge_point,
            commands::knowledge_point::move_knowledge_point,
            commands::knowledge_point::move_knowledge_point_to_course,
            commands::knowledge_point::delete_knowledge_point,
            commands::note::get_note,
            commands::note::save_note,
            commands::note::delete_note,
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::list_note_tags,
            commands::tag::attach_note_tag,
            commands::tag::detach_note_tag,
            commands::relation::list_relations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

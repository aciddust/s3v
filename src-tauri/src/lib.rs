mod error;
mod feedback;
pub mod logger;
mod profile;
mod s3;
mod state;
mod transfer;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            logger::init(app.handle().clone());
            let state =
                AppState::new(app.handle().clone()).expect("Failed to initialize app state");
            app.manage(state);
            logger::info("app", "S3V started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profile::commands::list_profiles,
            profile::commands::get_profile,
            profile::commands::create_profile,
            profile::commands::update_profile,
            profile::commands::delete_profile,
            profile::commands::test_connection,
            s3::commands::list_buckets,
            s3::commands::list_objects,
            s3::commands::head_object,
            s3::commands::delete_objects,
            s3::commands::rename_object,
            s3::commands::rename_folder,
            s3::commands::copy_object,
            s3::commands::move_objects,
            s3::commands::create_folder,
            s3::commands::get_presigned_url,
            s3::commands::list_multipart_uploads,
            s3::commands::abort_multipart_upload,
            transfer::commands::enqueue_upload,
            transfer::commands::enqueue_download,
            transfer::commands::pause_transfer,
            transfer::commands::resume_transfer,
            transfer::commands::cancel_transfer,
            transfer::commands::list_transfers,
            feedback::send_feedback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use tauri::{AppHandle, Manager, State, Emitter};
use crate::state::AppState;
use crate::types::*;
use crate::history::HistoryManager;
use crate::converter_impl::ComponentConverter;
use std::path::PathBuf;

/// Single component conversion
#[tauri::command]
pub async fn convert_component(
    lcsc_id: String,
    options: ConversionOptions,
    app: AppHandle,
    _state: State<'_, AppState>,
) -> std::result::Result<ConversionResult, String> {
    log::info!("Converting component: {}", lcsc_id);

    // Create converter
    let output_path = PathBuf::from(&options.output_dir);
    let converter = ComponentConverter::new(&output_path, options.kicad_v5);

    // Perform conversion
    let result = converter.convert(
        &lcsc_id,
        options.convert_symbol,
        options.convert_footprint,
        options.convert_3d,
        options.overwrite,
        options.target_kicad,
        options.target_altium,
    ).await;

    match result {
        Ok(conv_result) => {
            log::info!("Conversion successful: {}", conv_result.message);

            // Convert to API result type
            let api_result = ConversionResult {
                lcsc_id: conv_result.lcsc_id.clone(),
                success: conv_result.success,
                message: conv_result.message.clone(),
                component_name: conv_result.component_name.clone(),
                files_created: conv_result.files_created.clone(),
            };

            // Save to history
            if conv_result.success {
                if let Ok(app_dir) = app.path().app_data_dir() {
                    let _ = std::fs::create_dir_all(&app_dir);

                    if let Ok(history) = HistoryManager::new(app_dir) {
                        let _ = history.add_entry(&HistoryEntry {
                            id: 0,
                            lcsc_id: conv_result.lcsc_id,
                            component_name: conv_result.component_name,
                            success: conv_result.success,
                            timestamp: chrono::Local::now().to_rfc3339(),
                            output_dir: options.output_dir,
                        });
                    }
                }
            }

            Ok(api_result)
        }
        Err(e) => {
            let error_msg = format!("Conversion failed: {}", e);
            log::error!("{}", error_msg);

            Ok(ConversionResult {
                lcsc_id,
                success: false,
                message: error_msg,
                component_name: None,
                files_created: Vec::new(),
            })
        }
    }
}

/// Batch conversion (simplified)
#[tauri::command]
pub async fn batch_convert(
    lcsc_ids: Vec<String>,
    options: ConversionOptions,
    app: AppHandle,
    state: State<'_, AppState>,
) -> std::result::Result<BatchResult, String> {
    log::info!("Batch converting {} components", lcsc_ids.len());

    let total = lcsc_ids.len();
    let mut results = Vec::new();

    for (index, lcsc_id) in lcsc_ids.iter().enumerate() {
        let current = index + 1;

        // Emit progress: starting conversion
        let progress = ProgressUpdate {
            current,
            total,
            lcsc_id: lcsc_id.clone(),
            status: "converting".to_string(),
        };
        let _ = app.emit("conversion-progress", &progress);

        // Perform conversion
        let result = match convert_component(
            lcsc_id.clone(),
            options.clone(),
            app.clone(),
            state.clone(),
        ).await {
            Ok(r) => r,
            Err(e) => ConversionResult {
                lcsc_id: lcsc_id.clone(),
                success: false,
                message: e,
                component_name: None,
                files_created: Vec::new(),
            },
        };

        // Emit progress: completed or failed
        let status = if result.success { "completed" } else { "failed" };
        let progress = ProgressUpdate {
            current,
            total,
            lcsc_id: lcsc_id.clone(),
            status: status.to_string(),
        };
        let _ = app.emit("conversion-progress", &progress);

        results.push(result);
    }

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = total - succeeded;

    Ok(BatchResult {
        total,
        succeeded,
        failed,
        results,
    })
}

/// Select output directory
#[tauri::command]
pub async fn select_output_directory(app: AppHandle) -> std::result::Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app.dialog()
        .file()
        .blocking_pick_folder();

    match path {
        Some(p) => Ok(p.as_path().unwrap().display().to_string()),
        None => Err("No directory selected".to_string()),
    }
}

/// Get history
#[tauri::command]
pub async fn get_history(app: AppHandle, limit: usize) -> std::result::Result<Vec<HistoryEntry>, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;

    let _ = std::fs::create_dir_all(&app_dir);

    let history = HistoryManager::new(app_dir)
        .map_err(|e| format!("Failed to init history: {}", e))?;

    history.get_recent(limit)
        .map_err(|e| format!("Failed to fetch history: {}", e))
}

/// Clear history
#[tauri::command]
pub async fn clear_history(app: AppHandle) -> std::result::Result<(), String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;

    let history = HistoryManager::new(app_dir)
        .map_err(|e| format!("Failed to init history: {}", e))?;

    history.clear_all()
        .map_err(|e| format!("Failed to clear history: {}", e))
}

/// Import LCSC IDs from text file
#[tauri::command]
pub async fn import_ids_from_file(app: AppHandle) -> std::result::Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    // Open file picker for text files
    let path = app.dialog()
        .file()
        .add_filter("Text Files", &["txt"])
        .blocking_pick_file();

    match path {
        Some(p) => {
            let file_path = p.as_path().unwrap();

            // Read file content
            std::fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read file: {}", e))
        }
        None => Err("No file selected".to_string()),
    }
}

use tauri::{AppHandle, Manager, Emitter};
use crate::types::*;
use crate::history::HistoryManager;
use crate::converter_impl::ComponentConverter;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Single component conversion
#[tauri::command]
pub async fn convert_component(
    lcsc_id: String,
    options: ConversionOptions,
    app: AppHandle,
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

/// Batch conversion (parallel with semaphore-limited concurrency)
#[tauri::command]
pub async fn batch_convert(
    lcsc_ids: Vec<String>,
    options: ConversionOptions,
    app: AppHandle,
) -> std::result::Result<BatchResult, String> {
    log::info!("Batch converting {} components (max 4 concurrent)", lcsc_ids.len());

    let total = lcsc_ids.len();
    let semaphore = Arc::new(Semaphore::new(4));
    let completed = Arc::new(AtomicUsize::new(0));
    let mut join_set = JoinSet::new();

    for lcsc_id in lcsc_ids {
        let sem = semaphore.clone();
        let opts = options.clone();
        let handle = app.clone();
        let completed = completed.clone();
        let total = total;

        join_set.spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");

            let current = completed.fetch_add(1, Ordering::Relaxed) + 1;

            // Emit progress: starting conversion
            let _ = handle.emit("conversion-progress", &ProgressUpdate {
                current,
                total,
                lcsc_id: lcsc_id.clone(),
                status: "converting".to_string(),
            });

            // Perform conversion
            let output_path = PathBuf::from(&opts.output_dir);
            let converter = ComponentConverter::new(&output_path, opts.kicad_v5);

            let result = match converter.convert(
                &lcsc_id,
                opts.convert_symbol,
                opts.convert_footprint,
                opts.convert_3d,
                opts.overwrite,
            ).await {
                Ok(conv_result) => {
                    log::info!("Conversion successful: {}", conv_result.message);

                    // Save to history
                    if conv_result.success {
                        if let Ok(app_dir) = handle.path().app_data_dir() {
                            let _ = std::fs::create_dir_all(&app_dir);
                            if let Ok(history) = HistoryManager::new(app_dir) {
                                let _ = history.add_entry(&HistoryEntry {
                                    id: 0,
                                    lcsc_id: conv_result.lcsc_id.clone(),
                                    component_name: conv_result.component_name.clone(),
                                    success: conv_result.success,
                                    timestamp: chrono::Local::now().to_rfc3339(),
                                    output_dir: opts.output_dir.clone(),
                                });
                            }
                        }
                    }

                    ConversionResult {
                        lcsc_id: conv_result.lcsc_id,
                        success: conv_result.success,
                        message: conv_result.message,
                        component_name: conv_result.component_name,
                        files_created: conv_result.files_created,
                    }
                }
                Err(e) => {
                    let error_msg = format!("Conversion failed: {}", e);
                    log::error!("{}", error_msg);
                    ConversionResult {
                        lcsc_id: lcsc_id.clone(),
                        success: false,
                        message: error_msg,
                        component_name: None,
                        files_created: Vec::new(),
                    }
                }
            };

            // Emit progress: completed or failed
            let status = if result.success { "completed" } else { "failed" };
            let _ = handle.emit("conversion-progress", &ProgressUpdate {
                current,
                total,
                lcsc_id: result.lcsc_id.clone(),
                status: status.to_string(),
            });

            result
        });
    }

    // Collect all results
    let mut results = Vec::with_capacity(total);
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(result) => results.push(result),
            Err(e) => {
                log::error!("Task panicked: {}", e);
                results.push(ConversionResult {
                    lcsc_id: "unknown".to_string(),
                    success: false,
                    message: format!("Task panicked: {}", e),
                    component_name: None,
                    files_created: Vec::new(),
                });
            }
        }
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

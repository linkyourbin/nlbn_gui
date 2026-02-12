import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// Type definitions matching Rust types
interface ConversionOptions {
  output_dir: string;
  convert_symbol: boolean;
  convert_footprint: boolean;
  convert_3d: boolean;
  kicad_v5: boolean;
  project_relative: boolean;
  overwrite: boolean;
}

interface ConversionResult {
  lcsc_id: string;
  success: boolean;
  message: string;
  component_name: string | null;
  files_created: string[];
}

interface BatchResult {
  total: number;
  succeeded: number;
  failed: number;
  results: ConversionResult[];
}

interface HistoryEntry {
  id: number;
  lcsc_id: string;
  component_name: string | null;
  success: boolean;
  timestamp: string;
  output_dir: string;
}

interface ProgressUpdate {
  current: number;
  total: number;
  lcsc_id: string;
  status: string;
}

// DOM elements
let lcscIdInput: HTMLInputElement;
let outputDirInput: HTMLInputElement;
let convertBtn: HTMLButtonElement;
let selectDirBtn: HTMLButtonElement;
let importFileBtn: HTMLButtonElement;
let clearHistoryBtn: HTMLButtonElement;
let resultMessageDiv: HTMLElement;
let historyContainer: HTMLElement;

// Options checkboxes
let optSymbol: HTMLInputElement;
let optFootprint: HTMLInputElement;
let opt3D: HTMLInputElement;
let optOverwrite: HTMLInputElement;

// Progress listener cleanup
let progressUnlisten: UnlistenFn | null = null;
let progressMax = 0;

// Get conversion options from UI
function getConversionOptions(): ConversionOptions {
  return {
    output_dir: outputDirInput.value,
    convert_symbol: optSymbol.checked,
    convert_footprint: optFootprint.checked,
    convert_3d: opt3D.checked,
    kicad_v5: false,
    project_relative: false,
    overwrite: optOverwrite.checked,
  };
}

// Extract LCSC IDs from input text
// Supports comma-separated, space-separated, or newline-separated IDs
function extractLcscIds(input: string): string[] {
  // Match all patterns that start with C followed by digits
  const pattern = /C\d+/gi;
  const matches = input.match(pattern);

  if (!matches) {
    return [];
  }

  // Remove duplicates and convert to uppercase
  const uniqueIds = Array.from(new Set(matches.map(id => id.toUpperCase())));
  return uniqueIds;
}

// Show batch progress UI with progress bar
function showBatchProgressUI(_total: number) {
  progressMax = 0;
  resultMessageDiv.style.display = "block";
  resultMessageDiv.className = "result-item loading";
  resultMessageDiv.innerHTML = `
    <div class="batch-result-header">
      <h3>Converting Components...</h3>
    </div>
    <div class="batch-progress-bar">
      <div class="batch-progress-fill" id="live-progress-fill" style="width: 0%">
        <span class="batch-progress-text" id="live-progress-text">0%</span>
      </div>
    </div>
    <div id="live-progress-status" class="progress-status-text" style="font-size: 0.9rem; margin-top: 0.5rem;">
      Preparing...
    </div>
  `;
}

// Update batch progress in real-time
function updateBatchProgress(progress: ProgressUpdate) {
  // Track highest progress to prevent regression from out-of-order parallel events
  progressMax = Math.max(progressMax, progress.current);
  const percentage = Math.round((progressMax / progress.total) * 100);

  // Update button text
  convertBtn.textContent = `Converting... ${percentage}%`;

  // Update progress bar
  const progressFill = document.getElementById("live-progress-fill");
  const progressText = document.getElementById("live-progress-text");
  const progressStatus = document.getElementById("live-progress-status");

  if (progressFill) progressFill.style.width = `${percentage}%`;
  if (progressText) progressText.textContent = `${percentage}%`;

  if (progressStatus) {
    const statusIcon = progress.status === "completed" ? "✓" :
                      progress.status === "failed" ? "✗" : "⏳";
    progressStatus.textContent = `${statusIcon} ${progress.lcsc_id} — ${progress.status}`;
  }
}

// Convert component (single or batch)
async function convertComponent() {
  const input = lcscIdInput.value.trim();

  if (!input) {
    showResult({
      message: "Please enter an LCSC ID",
      success: false,
    });
    return;
  }

  // Extract all LCSC IDs from input
  const lcscIds = extractLcscIds(input);

  if (lcscIds.length === 0) {
    showResult({
      message: "No valid LCSC IDs found. IDs should start with 'C' followed by numbers (e.g., C529356)",
      success: false,
    });
    return;
  }

  // Disable button and show loading state
  convertBtn.disabled = true;
  showLoadingResult();

  try {
    const options = getConversionOptions();

    if (lcscIds.length === 1) {
      // Single component conversion
      convertBtn.textContent = "Converting...";
      const result: ConversionResult = await invoke("convert_component", {
        lcscId: lcscIds[0],
        options,
      });

      showResult(result);

      if (result.success) {
        await loadHistory();
      }
    } else {
      // Batch conversion
      try {
        // Setup progress listener
        progressUnlisten = await listen<ProgressUpdate>("conversion-progress", (event) => {
          updateBatchProgress(event.payload);
        });

        // Show initial progress UI
        showBatchProgressUI(lcscIds.length);

        // Start batch conversion
        const batchResult: BatchResult = await invoke("batch_convert", {
          lcscIds,
          options,
        });

        showBatchResult(batchResult);

        // Refresh history after batch conversion
        await loadHistory();
      } finally {
        // Clean up listener
        if (progressUnlisten) {
          progressUnlisten();
          progressUnlisten = null;
        }
      }
    }
  } catch (error) {
    showResult({
      lcsc_id: input,
      message: `Error: ${error}`,
      success: false,
      component_name: null,
      files_created: [],
    });
  } finally {
    convertBtn.disabled = false;
    convertBtn.textContent = "Convert";
  }
}

// Select output directory
async function selectOutputDirectory() {
  try {
    const path: string = await invoke("select_output_directory");
    outputDirInput.value = path;
  } catch (error) {
    console.log("Directory selection cancelled or failed:", error);
  }
}

// Import LCSC IDs from text file
async function importIdsFromFile() {
  try {
    const fileContent: string = await invoke("import_ids_from_file");

    // Extract IDs from file content using the same logic as manual input
    const ids = extractLcscIds(fileContent);

    if (ids.length === 0) {
      showResult({
        message: "No valid LCSC IDs found in file",
        success: false,
      });
      return;
    }

    // Fill the input with extracted IDs (comma-separated for clarity)
    lcscIdInput.value = ids.join(", ");

    // Show success message
    showResult({
      message: `Successfully imported ${ids.length} LCSC ID(s) from file`,
      success: true,
    });
  } catch (error) {
    console.log("File import cancelled or failed:", error);
  }
}

// Load and display history
async function loadHistory() {
  try {
    const history: HistoryEntry[] = await invoke("get_history", { limit: 50 });

    if (history.length === 0) {
      historyContainer.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">📋</div>
          <p>No conversion history yet</p>
        </div>
      `;
      return;
    }

    historyContainer.innerHTML = history
      .map((entry) => {
        const date = new Date(entry.timestamp);
        const timeStr = date.toLocaleString();
        const statusClass = entry.success ? "success" : "error";
        const statusIcon = entry.success ? "✓" : "✗";

        return `
          <div class="history-item">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <div>
                <strong>${entry.lcsc_id}</strong>
                ${entry.component_name ? `<span class="history-component-name"> - ${entry.component_name}</span>` : ""}
              </div>
              <span class="result-item-status status-${statusClass}">${statusIcon}</span>
            </div>
            <div class="history-item-time">${timeStr}</div>
            <div class="history-item-dir">
              ${entry.output_dir}
            </div>
          </div>
        `;
      })
      .join("");
  } catch (error) {
    console.error("Failed to load history:", error);
    historyContainer.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">⚠️</div>
        <p>Failed to load history</p>
      </div>
    `;
  }
}

// Clear history
async function clearHistory() {
  if (!confirm("Are you sure you want to clear all history?")) {
    return;
  }

  try {
    await invoke("clear_history");
    await loadHistory();
  } catch (error) {
    alert(`Failed to clear history: ${error}`);
  }
}

// Show batch result - only display failed components
function showBatchResult(batchResult: BatchResult) {
  resultMessageDiv.style.display = "block";

  const successRate = Math.round((batchResult.succeeded / batchResult.total) * 100);
  const failedResults = batchResult.results.filter(r => !r.success);

  // Set class based on overall success
  if (batchResult.failed === 0) {
    resultMessageDiv.className = "result-item batch-result success";
  } else if (batchResult.succeeded === 0) {
    resultMessageDiv.className = "result-item batch-result error";
  } else {
    resultMessageDiv.className = "result-item batch-result";
  }

  // Only show failed components list
  let failedListHtml = "";
  if (failedResults.length > 0) {
    failedListHtml = `
      <div class="batch-failed-section">
        <h4 style="color: var(--nlbn-error); font-size: 0.95rem; margin-bottom: 0.5rem;">
          ⚠️ Failed Components (${failedResults.length}):
        </h4>
        <div class="batch-results-list">
          ${failedResults
            .map((result) => {
              return `
                <div class="batch-result-item error">
                  <div class="batch-result-item-header">
                    <span class="batch-result-lcsc">${result.lcsc_id}</span>
                    <span class="result-item-status status-error">✗</span>
                  </div>
                  <div class="batch-result-message">${result.message}</div>
                </div>
              `;
            })
            .join("")}
        </div>
      </div>
    `;
  } else {
    // All succeeded - show success message
    failedListHtml = `
      <div style="text-align: center; padding: 0.5rem; color: var(--nlbn-success); font-size: 0.95rem;">
        🎉 All components converted successfully!
      </div>
    `;
  }

  resultMessageDiv.innerHTML = `
    <div class="batch-result-header">
      <h3>Batch Conversion Complete</h3>
      <div class="batch-stats">
        <span class="batch-stat-total">Total: ${batchResult.total}</span>
        <span class="batch-stat-success">✓ ${batchResult.succeeded}</span>
        ${batchResult.failed > 0 ? `<span class="batch-stat-failed">✗ ${batchResult.failed}</span>` : ""}
        <span class="batch-stat-rate">${successRate}%</span>
      </div>
    </div>
    <div class="batch-progress-bar">
      <div class="batch-progress-fill" style="width: ${successRate}%">
        <span class="batch-progress-text">${batchResult.succeeded} / ${batchResult.total}</span>
      </div>
    </div>
    ${failedListHtml}
  `;
}

// Show loading result
function showLoadingResult() {
  resultMessageDiv.style.display = "block";
  resultMessageDiv.className = "result-item loading";
  resultMessageDiv.innerHTML = `
    <div class="result-loading">
      <div class="loading-spinner"></div>
      <div class="result-item-message">Converting component...</div>
    </div>
  `;
}

// Show result message
function showResult(result: ConversionResult | { message: string; success: boolean }) {
  resultMessageDiv.style.display = "block";

  // Handle simple message format
  if ('message' in result && !('lcsc_id' in result)) {
    resultMessageDiv.className = `result-item ${result.success ? "success" : "error"}`;
    resultMessageDiv.innerHTML = `
      <div class="result-item-message">${result.message}</div>
    `;
    return;
  }

  // Handle full ConversionResult format
  const convResult = result as ConversionResult;
  const statusClass = convResult.success ? "success" : "error";
  const statusIcon = convResult.success ? "✓" : "✗";

  let filesHtml = "";
  if (convResult.files_created && convResult.files_created.length > 0) {
    filesHtml = `
      <div class="result-files">
        <div class="result-files-title">📁 Files Created:</div>
        <ul class="result-files-list">
          ${convResult.files_created
            .map(
              (file) => `<li title="${file}">${getFileName(file)}</li>`
            )
            .join("")}
        </ul>
      </div>
    `;
  }

  resultMessageDiv.className = `result-item ${statusClass}`;
  resultMessageDiv.innerHTML = `
    <div class="result-item-header">
      <div>
        <strong>${convResult.lcsc_id}</strong>
        ${
          convResult.component_name
            ? `<span class="result-component-name"> → ${convResult.component_name}</span>`
            : ""
        }
      </div>
      <span class="result-item-status status-${statusClass}">${statusIcon}</span>
    </div>
    <div class="result-item-message">${convResult.message}</div>
    ${filesHtml}
  `;
}

// Helper function to extract filename from full path
function getFileName(path: string): string {
  // Handle both Windows and Unix paths
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1];
}

// Initialize app
window.addEventListener("DOMContentLoaded", () => {
  // Get DOM elements
  lcscIdInput = document.querySelector("#lcsc-id")!;
  outputDirInput = document.querySelector("#output-dir")!;
  convertBtn = document.querySelector("#convert-btn")!;
  selectDirBtn = document.querySelector("#select-dir-btn")!;
  importFileBtn = document.querySelector("#import-file-btn")!;
  clearHistoryBtn = document.querySelector("#clear-history-btn")!;
  resultMessageDiv = document.querySelector("#result-message")!;
  historyContainer = document.querySelector("#history-container")!;;

  // Get option checkboxes
  optSymbol = document.querySelector("#opt-symbol")!;
  optFootprint = document.querySelector("#opt-footprint")!;
  opt3D = document.querySelector("#opt-3d")!;
  optOverwrite = document.querySelector("#opt-overwrite")!;

  // Event listeners
  convertBtn.addEventListener("click", convertComponent);
  selectDirBtn.addEventListener("click", selectOutputDirectory);
  importFileBtn.addEventListener("click", importIdsFromFile);
  clearHistoryBtn.addEventListener("click", clearHistory);

  // Allow Enter key to trigger conversion
  lcscIdInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      convertComponent();
    }
  });

  // Theme toggle
  const themeToggleBtn = document.querySelector("#theme-toggle-btn") as HTMLButtonElement;
  const savedTheme = localStorage.getItem("theme");
  const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const initialTheme = savedTheme ?? (systemDark ? "dark" : "light");

  document.documentElement.setAttribute("data-theme", initialTheme);
  themeToggleBtn.textContent = initialTheme === "dark" ? "☀️" : "🌙";

  themeToggleBtn.addEventListener("click", () => {
    const current = document.documentElement.getAttribute("data-theme");
    const next = current === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", next);
    localStorage.setItem("theme", next);
    themeToggleBtn.textContent = next === "dark" ? "☀️" : "🌙";
  });

  // Load history on startup
  loadHistory();
});

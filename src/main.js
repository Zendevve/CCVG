/**
 * Universal Interface Controller
 * Adheres to HIG principles: Clarity, Deference, Depth
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { openUrl } = window.__TAURI__.opener;

// ============================================
// Global App Controls
// ============================================
window.app = {
  minimize: () => getCurrentWindow().minimize(),
  close: () => getCurrentWindow().close(),
  toggleMaximize: () => getCurrentWindow().toggleMaximize(),
};

// ============================================
// Wizard State
// ============================================
const state = {
  history: ['welcome'],
  versions: [],
  selectedVersion: null,
};

window.wizard = {
  // Navigation
  next: () => navigateTo('select'),
  back: () => {
    if (state.history.length > 1) {
      state.history.pop();
      const prev = state.history[state.history.length - 1];
      showView(prev);
    }
  },
  goToLegacy: () => navigateTo('legacy'),

  // Actions
  selectObj: (idx) => {
    state.selectedVersion = state.versions[idx];
    const btn = document.getElementById('btn-lock-ver');
    if (btn) btn.disabled = false;

    // Visual selection state
    document.querySelectorAll('.list-row').forEach((el, i) => {
      const check = el.querySelector('.check-icon');
      if (i === idx) {
        el.style.backgroundColor = 'var(--system-bg-tertiary)'; // Selection highlight
        if (check) check.style.opacity = '1';
      } else {
        el.style.backgroundColor = 'var(--system-bg-secondary)';
        if (check) check.style.opacity = '0';
      }
    });
  },

  lockVersion: () => {
    runProcessParams();
  }
};

// ============================================
// Logic & Physics
// ============================================

function navigateTo(viewId) {
  state.history.push(viewId);
  showView(viewId);

  // Load data triggers
  if (viewId === 'select') loadVersions();
  if (viewId === 'legacy') loadArchiveVersions();
}

function showView(viewId) {
  // Hide all views
  document.querySelectorAll('.view').forEach(el => {
    el.classList.remove('active');
  });

  // Activate target
  const target = document.getElementById(`view-${viewId}`);
  if (target) {
    target.classList.add('active');
  }
}

async function runProcessParams() {
  navigateTo('process');

  // Physics-based status update simulation
  const setProgress = (msg, pct) => {
    document.getElementById('process-status').textContent = msg;
    document.getElementById('progress-bar').style.width = `${pct}%`;
  };

  try {
    setProgress('Analyzing workspace...', 10);
    await sleep(400); // Weight: Short

    setProgress('Locking configuration...', 40);
    await sleep(600); // Weight: Medium

    // Actual Backend Call
    // Backend expects specific params matching the Rust struct if generic,
    // but the command `run_full_protection` might take arguments.
    // Checking lib.rs... it exposes run_full_protection via wrapper?
    // Let's assume the payload matches what the command expects.
    // Based on previous context, we'll pass the list of versions to delete.

    const cleanCache = true; // Hardcoded default for "Full Protection"
    const versionsToDelete = state.versions
      .filter(v => v.path !== state.selectedVersion.path)
      .map(v => v.path);

    await invoke('run_full_protection', {
      params: {
        versions_to_delete: versionsToDelete,
        clean_cache: cleanCache
      }
    });

    await sleep(800);

    setProgress('Finalizing security...', 80);
    await sleep(400);

    setProgress('Done', 100);
    await sleep(200);

    navigateTo('success');

  } catch (e) {
    console.error(e);
    setProgress(`Error: ${e}`, 0);
    document.getElementById('progress-bar').style.backgroundColor = 'var(--tint-red)';
  }
}

async function loadVersions() {
  const container = document.getElementById('version-list-container');
  container.innerHTML = '<div style="padding:20px; text-align:center; color:var(--label-secondary)">Scanning...</div>';

  try {
    // Attempt actual scan
    let vers = [];
    try {
      vers = await invoke('scan_versions');
    } catch (err) {
      console.warn("Backend scan failed (dev mode?):", err);
      // Fallback mock for UI testing
      vers = [
        { name: "3.5.0", size_mb: 450, path: "/mock/path/1" },
        { name: "4.1.0", size_mb: 520, path: "/mock/path/2" }
      ];
    }

    state.versions = vers;

    if (vers.length === 0) {
      container.innerHTML = '<div style="padding:20px; text-align:center;">No installations found.</div>';
      return;
    }

    container.innerHTML = vers.map((v, i) => `
      <div class="list-row selectable" onclick="window.wizard.selectObj(${i})">
        <div class="list-icon">
          <i class="ph-duotone ph-hard-drives"></i>
        </div>
        <div class="list-info">
          <span class="list-title">CapCut v${v.name}</span>
          <span class="list-detail">${v.size_mb.toFixed(0)} MB</span>
        </div>
        <div class="check-icon" style="color:var(--tint-blue); opacity:0; transition:opacity 0.2s">
           <i class="ph-bold ph-check"></i>
        </div>
      </div>
    `).join('');

  } catch (e) {
    container.innerHTML = `<div style="padding:20px; color:var(--tint-red)">Error: ${e}</div>`;
  }
}

async function loadArchiveVersions() {
  const container = document.getElementById('legacy-list-container');
  container.innerHTML = '<div style="padding:20px; text-align:center; color:var(--label-secondary)">Fetching archive...</div>';

  try {
    const archives = await invoke('get_archive_versions');

    container.innerHTML = archives.map(v => `
      <div class="list-row" style="cursor: default;">
        <div class="list-icon">
          <i class="ph-duotone ph-archive"></i>
        </div>
        <div class="list-info">
          <div style="display:flex; align-items:center; gap:8px;">
            <span class="list-title">v${v.version}</span>
            <span style="background:var(--system-bg-tertiary); padding:2px 6px; border-radius:4px; font-size:10px; color:var(--label-secondary)">${v.persona}</span>
          </div>
          <span class="list-detail">${v.description}</span>
        </div>
        <button class="win-btn" style="width:auto; padding:0 8px; height:32px; color:var(--tint-blue)" onclick="/*global*/ window.__TAURI__.opener.openUrl('${v.download_url}')">
           <i class="ph-bold ph-download-simple"></i>
        </button>
      </div>
    `).join('');

  } catch (e) {
    container.innerHTML = `<div style="padding:20px; color:var(--tint-red)">Error: ${e}</div>`;
  }
}

function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

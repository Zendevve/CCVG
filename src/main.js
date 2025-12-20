/**
 * Universal Interface Controller
 * SwiftUI-like patterns with HIG compliance
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { openUrl } = window.__TAURI__.opener;

// ============================================
// App Controls (WindowGroup)
// ============================================
window.app = {
  minimize: () => getCurrentWindow().minimize(),
  close: () => getCurrentWindow().close(),
  toggleMaximize: () => getCurrentWindow().toggleMaximize(),
};

// ============================================
// State (ObservableObject)
// ============================================
const state = {
  history: ['welcome'],
  versions: [],
  selectedVersion: null,
  cacheEnabled: true,
  cacheSizeMb: 0,
};

// ============================================
// Wizard Actions
// ============================================
window.wizard = {
  startCheck: () => navigateTo('precheck'),
  next: () => navigateTo('select'),
  goToLegacy: () => navigateTo('legacy'),
  goToCleanup: () => navigateTo('cleanup'),

  back: () => {
    if (state.history.length > 1) {
      state.history.pop();
      showView(state.history[state.history.length - 1]);
    }
  },

  selectObj: (idx) => {
    state.selectedVersion = state.versions[idx];
    document.getElementById('btn-lock-ver').disabled = false;

    // Update selection visuals
    document.querySelectorAll('#version-list-container .list-row').forEach((el, i) => {
      const check = el.querySelector('.row-accessory');
      if (i === idx) {
        el.style.backgroundColor = 'var(--fill-tertiary)';
        if (check) check.style.opacity = '1';
      } else {
        el.style.backgroundColor = '';
        if (check) check.style.opacity = '0';
      }
    });
  },

  toggleCache: () => {
    state.cacheEnabled = !state.cacheEnabled;
    const toggle = document.getElementById('cache-toggle');
    toggle.classList.toggle('on', state.cacheEnabled);
  },

  runProtection: () => runProtectionSequence()
};

// ============================================
// Navigation (NavigationStack)
// ============================================
function navigateTo(viewId) {
  state.history.push(viewId);
  showView(viewId);

  // Trigger data loading
  if (viewId === 'precheck') runPreCheck();
  if (viewId === 'select') loadVersions();
  if (viewId === 'legacy') loadArchiveVersions();
  if (viewId === 'cleanup') loadCacheSize();
}

function showView(viewId) {
  document.querySelectorAll('.view').forEach(el => el.classList.remove('active'));
  const target = document.getElementById(`view-${viewId}`);
  if (target) target.classList.add('active');
}

// ============================================
// Pre-Check Logic
// ============================================
async function runPreCheck() {
  const installIcon = document.getElementById('check-install-icon');
  const installText = document.getElementById('check-install-text');
  const runningIcon = document.getElementById('check-running-icon');
  const runningText = document.getElementById('check-running-text');
  const nextBtn = document.getElementById('btn-precheck-next');

  // Reset
  setStatus(installIcon, 'pending');
  installText.textContent = 'Checking installation...';
  setStatus(runningIcon, 'pending');
  runningText.textContent = 'Checking processes...';
  nextBtn.disabled = true;

  await sleep(400);

  try {
    const result = await invoke('perform_precheck');

    // Installation status
    if (result.capcut_found) {
      setStatus(installIcon, 'success');
      installText.textContent = 'CapCut installation found';
    } else {
      setStatus(installIcon, 'error');
      installText.textContent = 'CapCut not found';
    }

    // Running status
    if (result.capcut_running) {
      setStatus(runningIcon, 'warning');
      runningText.textContent = 'CapCut is running — close it first';
    } else {
      setStatus(runningIcon, 'success');
      runningText.textContent = 'CapCut is not running';
    }

    // Enable continue if ready
    if (result.capcut_found && !result.capcut_running) {
      nextBtn.disabled = false;
    }
  } catch (e) {
    setStatus(installIcon, 'error');
    installText.textContent = `Error: ${e}`;
  }
}

function setStatus(icon, status) {
  icon.className = 'status-icon';
  const icons = {
    pending: 'ph-fill ph-circle-notch',
    success: 'ph-fill ph-check-circle',
    warning: 'ph-fill ph-warning',
    error: 'ph-fill ph-x-circle'
  };
  icon.classList.add(...icons[status].split(' '), status);
}

// ============================================
// Cache Size
// ============================================
async function loadCacheSize() {
  const sizeText = document.getElementById('cache-size-text');
  try {
    const size = await invoke('calculate_cache_size');
    state.cacheSizeMb = size;
    sizeText.textContent = `${size.toFixed(1)} MB can be freed`;
  } catch {
    sizeText.textContent = 'Size unavailable';
  }
}

// ============================================
// Protection Sequence
// ============================================
async function runProtectionSequence() {
  navigateTo('process');

  const progressBar = document.getElementById('progress-bar');
  const statusText = document.getElementById('process-status');
  const logContainer = document.getElementById('activity-log');
  logContainer.innerHTML = '';

  const setProgress = (msg, pct) => {
    statusText.textContent = msg;
    progressBar.style.width = `${pct}%`;
  };

  const addLog = (msg, type = 'info') => {
    const icons = { ok: 'ph-check', warn: 'ph-warning', info: 'ph-dot' };
    const colors = { ok: 'var(--tint-green)', warn: 'var(--tint-orange)', info: 'var(--label-tertiary)' };
    logContainer.innerHTML += `
      <div class="log-entry">
        <i class="ph ${icons[type]}" style="color: ${colors[type]};"></i>
        <span>${msg}</span>
      </div>
    `;
    logContainer.scrollTop = logContainer.scrollHeight;
  };

  try {
    setProgress('Preparing...', 10);
    addLog('Starting protection sequence');
    await sleep(300);

    const versionsToDelete = state.versions
      .filter(v => v.path !== state.selectedVersion.path)
      .map(v => v.path);

    setProgress('Cleaning versions...', 30);
    addLog(`Found ${versionsToDelete.length} version(s) to remove`);
    await sleep(200);

    setProgress('Applying protection...', 50);

    const result = await invoke('run_full_protection', {
      params: {
        versions_to_delete: versionsToDelete,
        clean_cache: state.cacheEnabled
      }
    });

    // Log backend output
    if (result.logs) {
      result.logs.forEach(log => {
        const type = log.startsWith('[OK]') ? 'ok' : log.startsWith('[!]') ? 'warn' : 'info';
        addLog(log.replace(/^\[OK\] |\[!\] |>> /g, ''), type);
      });
    }

    if (!result.success) {
      throw new Error(result.error || 'Protection failed');
    }

    setProgress('Finalizing...', 90);
    await sleep(300);

    setProgress('Complete', 100);
    addLog('Protection applied successfully', 'ok');
    await sleep(400);

    navigateTo('success');

  } catch (e) {
    console.error(e);
    document.getElementById('error-message').textContent = String(e);
    navigateTo('error');
  }
}

// ============================================
// Version List
// ============================================
async function loadVersions() {
  const container = document.getElementById('version-list-container');
  container.innerHTML = '<div class="list-row"><span class="row-title" style="color: var(--label-tertiary);">Scanning...</span></div>';

  try {
    const vers = await invoke('scan_versions');
    state.versions = vers;

    if (vers.length === 0) {
      container.innerHTML = '<div class="list-row"><span class="row-title">No installations found</span></div>';
      return;
    }

    container.innerHTML = vers.map((v, i) => `
      <div class="list-row selectable" onclick="window.wizard.selectObj(${i})">
        <div class="row-icon" style="background: var(--tint-indigo);">
          <i class="ph-fill ph-hard-drives"></i>
        </div>
        <div class="row-content">
          <span class="row-title">CapCut v${v.name}</span>
          <span class="row-subtitle">${v.size_mb.toFixed(0)} MB</span>
        </div>
        <i class="ph-bold ph-check row-accessory" style="opacity: 0; color: var(--tint-blue); font-size: 18px;"></i>
      </div>
    `).join('');

  } catch (e) {
    container.innerHTML = `<div class="list-row"><span class="row-title" style="color: var(--tint-red);">Error: ${e}</span></div>`;
  }
}

// ============================================
// Archive Versions
// ============================================
async function loadArchiveVersions() {
  const container = document.getElementById('legacy-list-container');
  container.innerHTML = '<div class="list-row"><span class="row-title" style="color: var(--label-tertiary);">Loading...</span></div>';

  try {
    const archives = await invoke('get_archive_versions');

    container.innerHTML = archives.map(v => {
      const riskColor = v.risk_level === 'High' ? 'var(--tint-red)' :
        v.risk_level === 'Medium' ? 'var(--tint-orange)' : 'var(--tint-green)';
      return `
        <div class="list-row">
          <div class="row-icon" style="background: ${riskColor};">
            <i class="ph-fill ph-package"></i>
          </div>
          <div class="row-content">
            <span class="row-title">v${v.version} · ${v.persona}</span>
            <span class="row-subtitle">${v.description}</span>
          </div>
          <button class="btn-plain" style="padding: 8px;" onclick="window.__TAURI__.opener.openUrl('${v.download_url}')">
            <i class="ph-bold ph-download-simple" style="font-size: 18px;"></i>
          </button>
        </div>
      `;
    }).join('');

  } catch (e) {
    container.innerHTML = `<div class="list-row"><span class="row-title" style="color: var(--tint-red);">Error: ${e}</span></div>`;
  }
}

// ============================================
// Utilities
// ============================================
function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

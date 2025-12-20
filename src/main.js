/**
 * Version Guard - Main Controller
 * macOS 26 Tahoe Design System
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

// ============================================
// Window Controls
// ============================================
document.getElementById('btn-close')?.addEventListener('click', () => getCurrentWindow().close());
document.getElementById('btn-minimize')?.addEventListener('click', () => getCurrentWindow().minimize());
document.getElementById('btn-maximize')?.addEventListener('click', () => getCurrentWindow().toggleMaximize());

// ============================================
// State
// ============================================
const state = {
  history: ['welcome'],
  versions: [],
  selectedVersion: null,
  cacheEnabled: true,
  lockEnabled: true,
  blockerEnabled: true,
  cacheSizeMb: 0,
};

// ============================================
// Navigation
// ============================================
function navigateTo(viewId) {
  state.history.push(viewId);
  showView(viewId);

  // Trigger data loading based on view
  if (viewId === 'precheck') runPreCheck();
  if (viewId === 'versions') loadVersions();
  if (viewId === 'legacy') loadArchiveVersions();
  if (viewId === 'options') loadCacheSize();
}

function showView(viewId) {
  document.querySelectorAll('.view').forEach(el => el.classList.remove('active'));
  const target = document.getElementById(`view-${viewId}`);
  if (target) target.classList.add('active');
}

function goBack() {
  if (state.history.length > 1) {
    state.history.pop();
    showView(state.history[state.history.length - 1]);
  }
}

// ============================================
// Welcome View Handlers
// ============================================
document.getElementById('btn-start')?.addEventListener('click', () => navigateTo('precheck'));
document.getElementById('btn-legacy')?.addEventListener('click', () => navigateTo('legacy'));

// ============================================
// Precheck View Handlers
// ============================================
document.getElementById('precheck-back')?.addEventListener('click', goBack);
document.getElementById('btn-continue-precheck')?.addEventListener('click', () => navigateTo('versions'));

async function runPreCheck() {
  const installIcon = document.getElementById('check-install');
  const installText = document.getElementById('check-install-text');
  const processIcon = document.getElementById('check-process');
  const processText = document.getElementById('check-process-text');
  const nextBtn = document.getElementById('btn-continue-precheck');

  // Reset
  setStatusIcon(installIcon, 'pending');
  installText.textContent = 'Checking installation...';
  setStatusIcon(processIcon, 'pending');
  processText.textContent = 'Checking processes...';
  nextBtn.disabled = true;

  await sleep(400);

  try {
    const result = await invoke('perform_precheck');

    if (result.capcut_found) {
      setStatusIcon(installIcon, 'success');
      installText.textContent = 'CapCut installation found';
    } else {
      setStatusIcon(installIcon, 'error');
      installText.textContent = 'CapCut not found';
    }

    if (result.capcut_running) {
      setStatusIcon(processIcon, 'warning');
      processText.textContent = 'CapCut is running — close it first';
    } else {
      setStatusIcon(processIcon, 'success');
      processText.textContent = 'CapCut is not running';
    }

    if (result.capcut_found && !result.capcut_running) {
      nextBtn.disabled = false;
    }
  } catch (e) {
    setStatusIcon(installIcon, 'error');
    installText.textContent = `Error: ${e}`;
  }
}

function setStatusIcon(icon, status) {
  icon.className = 'status-icon ph';
  const icons = {
    pending: 'ph-circle-notch spin',
    success: 'ph-check-circle',
    warning: 'ph-warning',
    error: 'ph-x-circle'
  };
  icon.classList.add(...icons[status].split(' '), status);
}

// ============================================
// Versions View Handlers
// ============================================
document.getElementById('versions-back')?.addEventListener('click', goBack);
document.getElementById('btn-continue-version')?.addEventListener('click', () => navigateTo('options'));

async function loadVersions() {
  const container = document.getElementById('version-list');
  container.innerHTML = '<div class="list-row"><span class="row-title" style="color: var(--label-tertiary);">Scanning...</span></div>';

  try {
    const vers = await invoke('scan_versions');
    state.versions = vers;

    if (vers.length === 0) {
      container.innerHTML = '<div class="list-row"><span class="row-title">No installations found</span></div>';
      return;
    }

    container.innerHTML = vers.map((v, i) => `
      <div class="list-row selectable" onclick="selectVersion(${i})">
        <div class="row-icon bg-accent-indigo">
          <i class="ph ph-hard-drives"></i>
        </div>
        <div class="row-content">
          <span class="row-title">CapCut v${v.name}</span>
          <span class="row-subtitle">${v.size_mb.toFixed(0)} MB</span>
        </div>
        <i class="ph ph-check row-accessory" style="opacity: 0; color: var(--accent-blue); font-size: 18px;"></i>
      </div>
    `).join('');

  } catch (e) {
    container.innerHTML = `<div class="list-row"><span class="row-title" style="color: var(--accent-red);">Error: ${e}</span></div>`;
  }
}

window.selectVersion = function (idx) {
  state.selectedVersion = state.versions[idx];
  document.getElementById('btn-continue-version').disabled = false;

  document.querySelectorAll('#version-list .list-row').forEach((el, i) => {
    const check = el.querySelector('.row-accessory');
    if (i === idx) {
      el.style.backgroundColor = 'var(--fill-tertiary)';
      if (check) check.style.opacity = '1';
    } else {
      el.style.backgroundColor = '';
      if (check) check.style.opacity = '0';
    }
  });
};

// ============================================
// Options View Handlers
// ============================================
document.getElementById('options-back')?.addEventListener('click', goBack);
document.getElementById('btn-apply')?.addEventListener('click', () => runProtectionSequence());

document.getElementById('toggle-cache')?.addEventListener('click', function () {
  state.cacheEnabled = !state.cacheEnabled;
  this.classList.toggle('on', state.cacheEnabled);
});

document.getElementById('toggle-lock')?.addEventListener('click', function () {
  state.lockEnabled = !state.lockEnabled;
  this.classList.toggle('on', state.lockEnabled);
});

document.getElementById('toggle-blocker')?.addEventListener('click', function () {
  state.blockerEnabled = !state.blockerEnabled;
  this.classList.toggle('on', state.blockerEnabled);
});

async function loadCacheSize() {
  const sizeText = document.getElementById('cache-size');
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
  navigateTo('processing');

  const progressBar = document.getElementById('progress-bar');
  const statusText = document.getElementById('status-text');
  const logContainer = document.getElementById('activity-log');
  logContainer.innerHTML = '';

  const setProgress = (msg, pct) => {
    statusText.textContent = msg;
    progressBar.style.width = `${pct}%`;
  };

  const addLog = (msg, type = 'info') => {
    const icons = { ok: 'ph-check', warn: 'ph-warning', info: 'ph-dot' };
    const classes = { ok: 'success', warn: 'warning', info: '' };
    logContainer.innerHTML += `
      <div class="log-entry ${classes[type]}">
        <i class="ph ${icons[type]}"></i>
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

    navigateTo('complete');

  } catch (e) {
    console.error(e);
    document.getElementById('error-message').textContent = String(e);
    navigateTo('error');
  }
}

// ============================================
// Complete View Handlers
// ============================================
document.getElementById('btn-done')?.addEventListener('click', () => getCurrentWindow().close());

// ============================================
// Error View Handlers
// ============================================
document.getElementById('btn-retry')?.addEventListener('click', () => {
  state.history = ['welcome'];
  showView('welcome');
});
document.getElementById('btn-back-error')?.addEventListener('click', goBack);

// ============================================
// Legacy View Handlers
// ============================================
document.getElementById('legacy-back')?.addEventListener('click', goBack);

async function loadArchiveVersions() {
  const container = document.getElementById('legacy-list');
  container.innerHTML = '<div class="list-row"><span class="row-title" style="color: var(--label-tertiary);">Loading...</span></div>';

  try {
    const archives = await invoke('get_archive_versions');

    container.innerHTML = archives.map(v => {
      const riskColor = v.risk_level === 'High' ? 'var(--accent-red)' :
        v.risk_level === 'Medium' ? 'var(--accent-orange)' : 'var(--accent-green)';
      return `
        <div class="list-row">
          <div class="row-icon" style="background: ${riskColor};">
            <i class="ph ph-package"></i>
          </div>
          <div class="row-content">
            <span class="row-title">v${v.version} · ${v.persona}</span>
            <span class="row-subtitle">${v.description}</span>
          </div>
          <button class="btn-plain" style="padding: 8px;" onclick="window.__TAURI__.opener.openUrl('${v.download_url}')">
            <i class="ph ph-download-simple" style="font-size: 18px;"></i>
          </button>
        </div>
      `;
    }).join('');

  } catch (e) {
    container.innerHTML = `<div class="list-row"><span class="row-title" style="color: var(--accent-red);">Error: ${e}</span></div>`;
  }
}

// ============================================
// Utilities
// ============================================
function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

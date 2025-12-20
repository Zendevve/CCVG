/**
 * Version Guard - Main Controller
 * macOS 26 Tahoe Design System
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { getVersion } = window.__TAURI__.app;

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
  switchTarget: null,
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
  if (viewId === 'switch') loadSwitchVersions();
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
document.getElementById('btn-remove-protection')?.addEventListener('click', removeProtection);

// Load protection status on start
(async function checkProtectionOnLoad() {
  try {
    const status = await invoke('check_protection_status');
    const badge = document.getElementById('protection-status');
    const removeBtn = document.getElementById('btn-remove-protection');

    if (status.is_protected) {
      badge.style.display = 'inline-flex';
      removeBtn.style.display = 'inline-flex';
    }
  } catch (e) {
    console.warn('Could not check protection status:', e);
  }
})();

async function removeProtection() {
  const btn = document.getElementById('btn-remove-protection');
  btn.disabled = true;
  btn.innerHTML = '<i class="ph ph-circle-notch spin"></i> Removing...';

  try {
    const result = await invoke('remove_protection');

    if (result.success) {
      btn.innerHTML = '<i class="ph ph-check"></i> Removed!';
      btn.style.background = 'var(--accent-green)';

      // Hide status badge
      document.getElementById('protection-status').style.display = 'none';

      await sleep(1500);
      btn.style.display = 'none';
    } else {
      throw new Error(result.error);
    }
  } catch (e) {
    btn.innerHTML = '<i class="ph ph-x"></i> Failed';
    btn.style.background = 'var(--accent-red)';
    console.error(e);
    await sleep(2000);
    btn.innerHTML = '<i class="ph ph-shield-slash"></i> Remove Protection';
    btn.style.background = '';
    btn.disabled = false;
  }
}

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
  const continueBtn = document.getElementById('btn-continue-version');

  // Reset state
  state.selectedVersion = null;
  continueBtn.disabled = true;

  // Show skeleton loader (Victor's Tips: use skeletons for layout)
  container.innerHTML = createSkeletonRows(3);

  try {
    const vers = await invoke('scan_versions');
    state.versions = vers;

    if (vers.length === 0) {
      container.innerHTML = `
        <div class="list-row" style="flex-direction: column; text-align: center; padding: 24px;">
          <i class="ph ph-folder-open" style="font-size: 32px; color: var(--label-tertiary); margin-bottom: 8px;"></i>
          <span class="row-title">No installations found</span>
          <span class="row-subtitle">Try downloading a legacy version first</span>
        </div>
      `;
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
        clean_cache: state.cacheEnabled,
        lock_config: state.lockEnabled,
        create_blockers: state.blockerEnabled
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
  container.innerHTML = createSkeletonRows(4);

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
// Quick Switch View Handlers
// ============================================
document.getElementById('btn-switch')?.addEventListener('click', () => navigateTo('switch'));
document.getElementById('switch-back')?.addEventListener('click', goBack);
document.getElementById('btn-switch-apply')?.addEventListener('click', applySwitch);

async function loadSwitchVersions() {
  const container = document.getElementById('switch-list');
  container.innerHTML = createSkeletonRows(2);

  try {
    const vers = await invoke('scan_versions');
    state.versions = vers;
    state.switchTarget = null;

    if (vers.length === 0) {
      container.innerHTML = `
        <div class="list-row" style="flex-direction: column; text-align: center; padding: 24px;">
          <i class="ph ph-folder-open" style="font-size: 32px; color: var(--label-tertiary); margin-bottom: 8px;"></i>
          <span class="row-title">No installations found</span>
        </div>
      `;
      return;
    }

    if (vers.length === 1) {
      container.innerHTML = '<div class="list-row"><span class="row-title">Only one version installed — nothing to switch</span></div>';
      return;
    }

    container.innerHTML = vers.map((v, i) => `
      <div class="list-row selectable" onclick="selectSwitchVersion(${i})">
        <div class="row-icon bg-accent-purple">
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

window.selectSwitchVersion = function (idx) {
  state.switchTarget = state.versions[idx];
  document.getElementById('btn-switch-apply').disabled = false;

  document.querySelectorAll('#switch-list .list-row').forEach((el, i) => {
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

async function applySwitch() {
  if (!state.switchTarget) return;

  const btn = document.getElementById('btn-switch-apply');
  btn.disabled = true;
  btn.innerHTML = '<i class="ph ph-circle-notch spin"></i> Switching...';

  try {
    const result = await invoke('switch_version', { targetPath: state.switchTarget.path });

    if (result.success) {
      btn.innerHTML = '<i class="ph ph-check"></i> Switched!';
      btn.style.background = 'var(--accent-green)';
      await sleep(1000);
      state.history = ['welcome'];
      showView('welcome');
      btn.innerHTML = '<i class="ph ph-swap"></i> Switch Version';
      btn.style.background = '';
    } else {
      throw new Error(result.message);
    }
  } catch (e) {
    btn.innerHTML = '<i class="ph ph-x"></i> Failed';
    btn.style.background = 'var(--accent-red)';
    console.error(e);
    await sleep(2000);
    btn.innerHTML = '<i class="ph ph-swap"></i> Switch Version';
    btn.style.background = '';
    btn.disabled = false;
  }
}

// ============================================
// Utilities
// ============================================
function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

// Create skeleton loading rows (Victor's Tips: use skeletons for layout)
function createSkeletonRows(count = 3) {
  return Array(count).fill(0).map(() => `
    <div class="skeleton-row">
      <div class="skeleton-icon"></div>
      <div class="skeleton-text">
        <div class="skeleton-line medium"></div>
        <div class="skeleton-line short"></div>
      </div>
    </div>
  `).join('');
}

// ============================================
// Dynamic Version Display
// ============================================
(async function loadAppVersion() {
  try {
    const version = await getVersion();
    const versionEl = document.getElementById('app-version');
    if (versionEl) versionEl.textContent = `v${version}`;
  } catch (e) {
    console.warn('Could not load app version:', e);
  }
})();

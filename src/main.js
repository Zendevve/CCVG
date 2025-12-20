/**
 * CC Version Guard - Wizard Mode
 * Feature parity with legacy eframe/egui app
 */

const { invoke } = window.__TAURI__.core;
const { openUrl } = window.__TAURI__.opener;
const { getCurrentWindow } = window.__TAURI__.window; // Import window API

// ============================================
// State
// ============================================
const state = {
  currentScreen: 'welcome',
  currentStep: 1,
  installedVersions: [],
  archiveVersions: [],
  selectedVersion: null,
  cacheSize: 0,
  cleanCache: true,
};

// ============================================
// Initialization
// ============================================
document.addEventListener('DOMContentLoaded', async () => {
  // Window Controls Logic
  const appWindow = getCurrentWindow();
  document.getElementById('win-minimize').addEventListener('click', () => appWindow.minimize());
  document.getElementById('win-maximize').addEventListener('click', () => appWindow.toggleMaximize());
  document.getElementById('win-close').addEventListener('click', () => appWindow.close());
  // Load archive versions for download manager
  try {
    state.archiveVersions = await invoke('get_archive_versions');
  } catch (e) {
    console.error('Failed to load archive versions:', e);
  }

  renderDownloadManager();
  updateProgressBar();
});

// ============================================
// Screen Navigation
// ============================================
// ============================================
// Screen Navigation
// ============================================
function showScreen(screenId) {
  document.querySelectorAll('.screen').forEach(el => el.classList.remove('active'));
  const target = document.getElementById(`screen-${screenId}`);
  if (target) target.classList.add('active');
  state.currentScreen = screenId;

  // Toggle progress bar visibility
  const progressBar = document.querySelector('.progress-bar');
  if (screenId === 'download-manager') {
    progressBar.style.display = 'none';
  } else {
    progressBar.style.display = 'flex';
  }

  // Update progress bar step
  if (screenId === 'welcome') state.currentStep = 1;
  if (screenId === 'version-select') state.currentStep = 2;
  if (screenId === 'cache-clean') state.currentStep = 3;
  if (screenId === 'complete') state.currentStep = 4;

  updateProgressBar();
}

function updateProgressBar() {
  document.querySelectorAll('.progress-step').forEach(el => {
    const step = parseInt(el.dataset.step);
    el.classList.remove('active', 'completed');

    if (step === state.currentStep) {
      el.classList.add('active');
    } else if (step < state.currentStep) {
      el.classList.add('completed');
    }
  });
}

// ============================================
// Download Manager
// ============================================
function renderDownloadManager() {
  const grid = document.getElementById('version-grid');
  if (!grid) return;

  const getIcon = (persona) => {
    if (persona.includes('Audio')) return 'ph-waves';
    if (persona.includes('Classic')) return 'ph-film-strip';
    if (persona.includes('Stable')) return 'ph-shield-check';
    return 'ph-download-simple';
  };

  grid.innerHTML = state.archiveVersions.map(v => `
    <div class="version-card">
      <div class="card-header">
        <div class="icon-glow-ring small accent">
            <i class="ph-fill ${getIcon(v.persona)}"></i>
        </div>
        <div>
            <div class="persona">${v.persona}</div>
            <div class="ver">v${v.version}</div>
        </div>
      </div>
      <div class="desc">${v.description}</div>
      <button class="dl-btn" onclick="downloadVersion('${v.download_url}')">
        <i class="ph-bold ph-download-simple"></i> Download
      </button>
    </div>
  `).join('');
}

function downloadVersion(url) {
  openUrl(url);
}

// ============================================
// PreCheck Flow
// ============================================
async function startCheck() {
  showScreen('precheck');

  try {
    // Check if CapCut is running
    const running = await invoke('is_capcut_running');
    if (running) {
      showScreen('running-warning');
      return;
    }

    // Scan for versions
    document.getElementById('precheck-status').textContent = 'Scanning for installed versions...';
    state.installedVersions = await invoke('scan_versions');

    if (state.installedVersions.length === 0) {
      showScreen('not-found');
      return;
    }

    // Load cache size
    try {
      state.cacheSize = await invoke('calculate_cache_size');
    } catch (e) {
      state.cacheSize = 0;
    }

    // Go to version select
    renderVersionSelect();
    showScreen('version-select');

  } catch (e) {
    console.error(e);
    document.getElementById('error-text').textContent = e.toString();
    showScreen('error');
  }
}

// ============================================
// Version Selection
// ============================================
function renderVersionSelect() {
  const container = document.getElementById('installed-versions');

  container.innerHTML = state.installedVersions.map((v, idx) => `
    <div class="version-option" data-idx="${idx}" onclick="selectVersion(${idx})">
      <div class="version-radio"></div>
      <div class="version-info">
        <div class="version-name">CapCut v${v.name}</div>
        <div class="version-meta">${v.size_mb.toFixed(0)} MB</div>
      </div>
    </div>
  `).join('');
}

function selectVersion(idx) {
  state.selectedVersion = state.installedVersions[idx];

  document.querySelectorAll('.version-option').forEach((el, i) => {
    el.classList.toggle('selected', i === idx);
  });

  document.getElementById('btn-continue-select').disabled = false;
}

function goToCacheScreen() {
  document.getElementById('cache-size').textContent = `${state.cacheSize.toFixed(1)} MB`;
  showScreen('cache-clean');
}

// ============================================
// Protection
// ============================================
async function startProtection() {
  state.cleanCache = document.getElementById('cache-toggle').checked;
  showScreen('running');

  const statusEl = document.getElementById('running-status');
  const progressEl = document.getElementById('progress-fill');
  const logEl = document.getElementById('action-log');

  logEl.innerHTML = '';

  function log(msg, isOk = false) {
    logEl.innerHTML += `<div class="${isOk ? 'ok' : ''}">${msg}</div>`;
    logEl.scrollTop = logEl.scrollHeight;
  }

  try {
    // Collect versions to delete
    const versionsToDelete = state.installedVersions
      .filter(v => v.path !== state.selectedVersion.path)
      .map(v => v.path);

    statusEl.textContent = 'Preparing...';
    progressEl.style.width = '10%';
    log('Starting protection sequence...');
    await sleep(300);

    // Call backend
    statusEl.textContent = 'Applying protection...';
    progressEl.style.width = '30%';
    log(`Keeping version: ${state.selectedVersion.name}`);

    if (versionsToDelete.length > 0) {
      log(`Removing ${versionsToDelete.length} other version(s)...`);
    }

    const result = await invoke('run_full_protection', {
      params: {
        versions_to_delete: versionsToDelete,
        clean_cache: state.cleanCache,
      }
    });

    progressEl.style.width = '80%';

    // Log results
    if (result.logs) {
      result.logs.forEach(l => {
        const isOk = l.includes('[OK]');
        log(l, isOk);
      });
    }

    if (!result.success) {
      throw new Error(result.error || 'Protection failed');
    }

    statusEl.textContent = 'Finalizing...';
    progressEl.style.width = '100%';
    await sleep(500);

    // Show/hide cache cleaned row
    const cacheRow = document.getElementById('cache-cleaned-row');
    cacheRow.style.display = state.cleanCache ? 'flex' : 'none';

    document.getElementById('complete-text').textContent =
      `CapCut v${state.selectedVersion.name} is now protected.`;

    showScreen('complete');

  } catch (e) {
    console.error(e);
    document.getElementById('error-text').textContent = e.toString();
    showScreen('error');
  }
}

// ============================================
// Utilities
// ============================================
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function openGitHub() {
  openUrl('https://github.com/Zendevve/capcut-version-guard');
}

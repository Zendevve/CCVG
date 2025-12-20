/**
 * CC Version Guard v3 - Project Onyx
 * Hybrid Wizard + Dashboard Architecture
 *
 * UX Principles Applied:
 * - Hick's Law: One action per screen in wizard
 * - Jakob's Law: Familiar setup wizard pattern
 * - Peak-End Rule: Delightful success animation
 * - Progressive Disclosure: Simple wizard → Advanced dashboard
 */

const { invoke } = window.__TAURI__.core;

// ============================================
// Global State
// ============================================
const state = {
  mode: 'wizard', // 'wizard' or 'dashboard'
  installedVersions: [],
  archiveVersions: [],
  selectedVersion: null,     // For wizard
  selectedDownload: null,    // For download wizard
  cacheSize: 0,
  precheck: { found: false, running: false, appsPath: null },
};

// ============================================
// Initialization
// ============================================
document.addEventListener('DOMContentLoaded', async () => {
  // Check if user prefers dashboard (power user)
  const preferDashboard = localStorage.getItem('preferDashboard') === 'true';

  if (preferDashboard) {
    enterDashboard();
  } else {
    startWizard();
  }
});

// ============================================
// WIZARD MODE
// ============================================

async function startWizard() {
  state.mode = 'wizard';
  wizardGoToStep(1);

  // Auto-scan after a brief delay (for visual feedback)
  await sleep(800);

  try {
    // Load archive versions (for download option)
    state.archiveVersions = await invoke('get_archive_versions');

    // Check if CapCut is running
    const running = await invoke('is_capcut_running');
    if (running) {
      wizardGoToStep('running');
      return;
    }

    // Scan for installed versions
    state.installedVersions = await invoke('scan_versions');

    if (state.installedVersions.length === 0) {
      wizardGoToStep('notfound');
    } else {
      renderWizardVersionList();
      wizardGoToStep(2);
    }
  } catch (e) {
    console.error(e);
    document.getElementById('wiz-error-text').textContent = e.toString();
    wizardGoToStep('error');
  }
}

function wizardGoToStep(step) {
  // Hide all wizard screens
  document.querySelectorAll('.wiz-screen').forEach(el => el.classList.remove('active'));

  // Show target screen
  const targetId = typeof step === 'number' ? `wiz-step-${step}` : `wiz-step-${step}`;
  const target = document.getElementById(targetId);
  if (target) target.classList.add('active');
}

function showWizDownload() {
  renderWizardDownloadList();
  wizardGoToStep('download');
}

function renderWizardVersionList() {
  const container = document.getElementById('wiz-version-list');

  container.innerHTML = state.installedVersions.map((v, idx) => `
    <div class="wiz-option" data-idx="${idx}" onclick="selectWizVersion(${idx})">
      <div class="wiz-option-radio"></div>
      <div class="wiz-option-info">
        <div class="wiz-option-name">CapCut v${v.name}</div>
        <div class="wiz-option-meta">${v.size_mb.toFixed(0)} MB</div>
      </div>
    </div>
  `).join('');
}

function selectWizVersion(idx) {
  state.selectedVersion = state.installedVersions[idx];

  // Update UI
  document.querySelectorAll('#wiz-version-list .wiz-option').forEach((el, i) => {
    el.classList.toggle('selected', i === idx);
  });

  // Enable button
  const btn = document.getElementById('wiz-protect-btn');
  btn.disabled = false;
  btn.innerHTML = `<i class="ph-bold ph-shield-check"></i><span>Protect v${state.selectedVersion.name}</span>`;
}

function renderWizardDownloadList() {
  const container = document.getElementById('wiz-download-list');

  container.innerHTML = state.archiveVersions.map((v, idx) => `
    <div class="wiz-option" data-idx="${idx}" onclick="selectWizDownload(${idx})">
      <div class="wiz-option-radio"></div>
      <div class="wiz-option-info">
        <div class="wiz-option-name">${v.persona} (v${v.version})</div>
        <div class="wiz-option-meta">${v.description}</div>
      </div>
    </div>
  `).join('');
}

function selectWizDownload(idx) {
  state.selectedDownload = state.archiveVersions[idx];

  document.querySelectorAll('#wiz-download-list .wiz-option').forEach((el, i) => {
    el.classList.toggle('selected', i === idx);
  });

  const btn = document.getElementById('wiz-download-btn');
  btn.disabled = false;
  btn.innerHTML = `<i class="ph-bold ph-download-simple"></i><span>Download v${state.selectedDownload.version}</span>`;
}

async function wizardDownload() {
  if (!state.selectedDownload) return;

  // Open download URL
  window.open(state.selectedDownload.download_url, '_blank');

  // Show instruction
  document.getElementById('wiz-error-text').textContent =
    'Download started! After installing, restart this app to protect your new version.';
  wizardGoToStep('error'); // Reusing error screen for info
}

async function wizardProtect() {
  if (!state.selectedVersion) return;

  wizardGoToStep(3);
  const progressText = document.getElementById('wiz-progress-text');
  const progressFill = document.getElementById('wiz-progress-fill');

  try {
    // Step 1: Collect versions to delete
    progressText.textContent = 'Analyzing versions...';
    progressFill.style.width = '20%';
    await sleep(400);

    const versionsToDelete = state.installedVersions
      .filter(v => v.path !== state.selectedVersion.path)
      .map(v => v.path);

    // Step 2: Apply protection
    progressText.textContent = 'Locking configuration...';
    progressFill.style.width = '50%';

    await invoke('run_full_protection', {
      appsPath: state.selectedVersion.path.replace(/\\[^\\]+$/, ''), // parent dir
      versionsToDelete: versionsToDelete,
      selectedVersionPath: state.selectedVersion.path,
      cleanCache: true,
    });

    progressText.textContent = 'Finalizing...';
    progressFill.style.width = '100%';
    await sleep(500);

    // Done!
    document.getElementById('wiz-done-text').textContent =
      `CapCut v${state.selectedVersion.name} is now protected.`;
    wizardGoToStep('done');

  } catch (e) {
    console.error(e);
    document.getElementById('wiz-error-text').textContent = e.toString();
    wizardGoToStep('error');
  }
}

function wizardRestart() {
  state.selectedVersion = null;
  state.selectedDownload = null;
  startWizard();
}

// ============================================
// DASHBOARD MODE
// ============================================

function enterDashboard() {
  state.mode = 'dashboard';
  localStorage.setItem('preferDashboard', 'true');

  document.getElementById('wizard-overlay').classList.add('hidden');
  document.getElementById('app-container').classList.remove('hidden');

  loadDashboard();
}

const router = {
  current: 'dashboard',
  navigate(viewId) {
    document.querySelectorAll('.nav-item').forEach(el => {
      const onclick = el.getAttribute('onclick') || '';
      el.classList.toggle('active', onclick.includes(viewId));
    });

    document.querySelectorAll('.view-section').forEach(el => {
      el.classList.remove('active');
    });

    const target = document.getElementById(`view-${viewId}`);
    if (target) target.classList.add('active');

    this.current = viewId;

    if (viewId === 'dashboard') loadDashboard();
    if (viewId === 'my-versions') loadInstalledVersions();
    if (viewId === 'cleaner') loadCleaner();
    if (viewId === 'library') renderLibrary();
  }
};

async function loadDashboard() {
  await loadInstalledVersions();
  await updateCacheCard();
  updateStatusOrb();
}

async function loadInstalledVersions() {
  try {
    state.installedVersions = await invoke('scan_versions');

    if (state.installedVersions.length > 0) {
      document.getElementById('dash-active-ver').textContent = state.installedVersions[0].name;
      document.getElementById('dash-install-count').textContent = state.installedVersions.length;
    }

    renderVersionsList();
    renderLibrary();
  } catch (e) {
    console.error(e);
  }
}

async function updateCacheCard() {
  try {
    state.cacheSize = await invoke('calculate_cache_size');
    document.getElementById('dash-cache-size').textContent = `${state.cacheSize.toFixed(1)} MB`;
    document.getElementById('cleaner-size').textContent = `${state.cacheSize.toFixed(1)} MB`;
  } catch (e) { }
}

function updateStatusOrb() {
  const orb = document.getElementById('status-orb');
  const text = document.getElementById('status-text');
  const sub = document.getElementById('status-sub');

  orb.className = 'status-orb protected';
  orb.innerHTML = '<i class="ph-fill ph-shield-check"></i>';
  text.textContent = 'System Protected';
  sub.textContent = 'Configuration locked • Blocker active';
}

function renderVersionsList() {
  const container = document.getElementById('installed-list');
  if (!container) return;

  if (state.installedVersions.length === 0) {
    container.innerHTML = '<p class="text-muted text-center mt-lg">No versions found.</p>';
    return;
  }

  container.innerHTML = state.installedVersions.map((v, idx) => {
    const isActive = idx === 0;
    return `
      <div class="version-row ${isActive ? 'active' : ''}">
        <i class="ph-fill ${isActive ? 'ph-check-circle' : 'ph-circle'} v-icon"></i>
        <div class="v-info">
          <div class="v-name">CapCut v${v.name}</div>
          <div class="v-meta">${v.size_mb.toFixed(1)} MB</div>
        </div>
        <div class="v-actions">
          ${!isActive ? `
            <button class="btn btn-primary" onclick="requestSwitch('${v.path.replace(/\\/g, '\\\\')}')">Switch To</button>
          ` : '<span class="text-success text-sm">Active</span>'}
        </div>
      </div>
    `;
  }).join('');
}

function renderLibrary() {
  const container = document.getElementById('library-grid');
  if (!container) return;

  container.innerHTML = state.archiveVersions.map(v => {
    const isInstalled = state.installedVersions.some(iv => iv.name.includes(v.version));
    return `
      <div class="card">
        <span class="card-title text-accent">${v.persona}</span>
        <div class="v-name mb-sm">v${v.version}</div>
        <p class="text-muted text-sm mb-md" style="flex:1">${v.description}</p>
        ${isInstalled ?
        '<button class="btn w-full" disabled>Installed</button>' :
        `<button class="btn btn-primary w-full" onclick="window.open('${v.download_url}', '_blank')">
            <i class="ph-bold ph-download-simple"></i> Download
          </button>`
      }
      </div>
    `;
  }).join('');
}

// ============================================
// Switcher & Modal (Shared)
// ============================================

let pendingSwitchTarget = null;

function requestSwitch(path) {
  pendingSwitchTarget = path;
  document.getElementById('safety-modal').classList.remove('hidden');
}

function closeModal() {
  document.getElementById('safety-modal').classList.add('hidden');
  pendingSwitchTarget = null;
}

async function confirmSwitch() {
  if (!pendingSwitchTarget) return;

  try {
    await invoke('switch_version', { targetPath: pendingSwitchTarget });
    closeModal();
    await loadDashboard();
  } catch (e) {
    alert('Switch failed: ' + e);
    closeModal();
  }
}

function openProjectsFolder() {
  // Open file explorer to CapCut projects folder
  invoke('get_capcut_paths').then(paths => {
    if (paths) {
      window.open(`file://${paths[1]}/User Data/Projects`, '_blank');
    }
  });
}

// ============================================
// Cleaner
// ============================================

async function loadCleaner() {
  await updateCacheCard();
}

async function runCleaner() {
  const btn = document.querySelector('#view-cleaner button');
  const original = btn.innerHTML;
  btn.textContent = 'Cleaning...';
  btn.disabled = true;

  try {
    await invoke('clean_cache');
    await loadDashboard();
    btn.innerHTML = '<i class="ph-bold ph-check"></i> Cleaned!';
    setTimeout(() => {
      btn.innerHTML = original;
      btn.disabled = false;
    }, 2000);
  } catch (e) {
    btn.textContent = 'Error';
  }
}

// ============================================
// Utilities
// ============================================

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

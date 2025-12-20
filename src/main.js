/**
 * CC Version Guard v3 - Project Onyx
 * Main Application Logic (Dashboard Architecture)
 */

const { invoke } = window.__TAURI__.core;
const { openUrl } = window.__TAURI__.opener;

// ============================================
// State
// ============================================
const state = {
  activeVersion: null,
  installedVersions: [],
  archiveVersions: [],
  cacheSize: 0,
  precheck: { found: false, running: false, appsPath: null },
  pendingSwitchTarget: null, // Path of version to switch to
};

// ============================================
// Router
// ============================================
const router = {
  current: 'dashboard',
  navigate(viewId) {
    // Update Sidebar
    document.querySelectorAll('.nav-item').forEach(el => {
      el.classList.toggle('active', el.getAttribute('onclick')?.includes(viewId));
    });

    // Update Views
    document.querySelectorAll('.view-section').forEach(el => {
      el.classList.remove('active');
    });
    const target = document.getElementById(`view-${viewId}`);
    if (target) target.classList.add('active');

    this.current = viewId;

    // View-specific reloads
    if (viewId === 'dashboard') loadDashboard();
    if (viewId === 'my-versions') loadInstalledVersions();
    if (viewId === 'cleaner') loadCleaner();
  }
};

// ============================================
// Initialization
// ============================================
document.addEventListener('DOMContentLoaded', async () => {
  await loadArchiveVersions();
  await loadInstalledVersions(); // Also refreshes dashboard state
  loadDashboard(); // Initial render

  // Background check
  checkSystemStatus();
});

async function checkSystemStatus() {
  try {
    const res = await invoke('perform_precheck');
    state.precheck = res;
    updateStatusOrb();
  } catch (e) {
    console.error("Precheck failed", e);
  }
}

// ============================================
// Dashboard Controller
// ============================================
async function loadDashboard() {
  // Update Cards
  if (state.installedVersions.length > 0) {
    // Assume 0 is active or find one (logic to be improved with accurate active detection)
    // For now, we rely on installedVersions[0] being the "active" one if we just protected it
    // In a real scenario we'd read configure.ini to know for sure.
    // For V3, let's assume the first one is active for UI or "Unknown"
    const active = state.installedVersions[0];
    document.getElementById('dash-active-ver').textContent = active ? active.name : "None";
    document.getElementById('dash-install-count').textContent = state.installedVersions.length;
  }

  // Cache
  updateCacheCard();

  // Precheck again
  checkSystemStatus();
}

async function updateCacheCard() {
  try {
    const size = await invoke('calculate_cache_size');
    state.cacheSize = size;
    document.getElementById('dash-cache-size').textContent = `${size.toFixed(1)} MB`;
    document.getElementById('cleaner-size').textContent = `${size.toFixed(1)} MB`;
  } catch (e) { }
}

function updateStatusOrb() {
  const orb = document.getElementById('status-orb');
  const text = document.getElementById('status-text');
  const sub = document.getElementById('status-sub');

  orb.className = 'status-orb'; // reset

  if (state.precheck.running) {
    orb.classList.add('risk');
    orb.innerHTML = '<i class="ph-fill ph-warning"></i>';
    text.textContent = "CapCut Running";
    sub.textContent = "Please close CapCut to manage versions";
  } else if (state.installedVersions.length > 0) {
    orb.classList.add('protected');
    orb.innerHTML = '<i class="ph-fill ph-shield-check"></i>';
    text.textContent = "System Protected";
    sub.textContent = "Configuration locked • Blocker active";
  } else {
    orb.innerHTML = '<i class="ph-fill ph-shield-slash"></i>';
    text.textContent = "No Installation";
    sub.textContent = "CapCut not found";
  }
}

// ============================================
// My Versions Manager
// ============================================
async function loadInstalledVersions() {
  try {
    const versions = await invoke('scan_versions');
    state.installedVersions = versions;
    renderVersionsList();
    renderLibrary(); // Re-render library to show "Installed" badges
  } catch (e) {
    console.error(e);
  }
}

function renderVersionsList() {
  const container = document.getElementById('installed-list');

  if (state.installedVersions.length === 0) {
    container.innerHTML = `<p class="text-muted text-center mt-lg">No versions found.</p>`;
    return;
  }

  container.innerHTML = state.installedVersions.map((v, idx) => {
    // Determine active (simplistic logic for now, assumes first in scanned list is intended active or we need to read config)
    // For ONYX v1, we will treat the first version as "Active" for visual purposes or add logic to read last_version from backend
    const isActive = idx === 0;

    return `
      <div class="version-row ${isActive ? 'active' : ''}">
        <i class="ph-fill ${isActive ? 'ph-check-circle' : 'ph-circle'} v-icon"></i>
        <div class="v-info">
          <div class="v-name">CapCut v${v.name}</div>
          <div class="v-meta">${v.size_mb.toFixed(1)} MB • ${v.path}</div>
        </div>
        <div class="v-actions">
          ${!isActive ? `
            <button class="btn btn-primary" onclick="requestSwitch('${v.path.replace(/\\/g, '\\\\')}')">
              Switch To
            </button>
            <button class="btn btn-danger" onclick="deleteVersion('${v.path.replace(/\\/g, '\\\\')}')">
              Uninstall
            </button>
          ` : `<span class="text-success text-sm font-bold" style="padding: 8px">Active</span>`}
        </div>
      </div>
    `;
  }).join('');
}

// ============================================
// Version Switcher (Non-Destructive)
// ============================================
function requestSwitch(path) {
  state.pendingSwitchTarget = path;

  // Show Warning Modal
  const modal = document.getElementById('safety-modal');
  modal.classList.remove('hidden');
}

function closeModal() {
  const modal = document.getElementById('safety-modal');
  modal.classList.add('hidden');
  state.pendingSwitchTarget = null;
}

async function openProjectsFolder() {
  try {
    // We assume standard location or ask backend for it
    // For simplicity, launch user directory
    await openUrl('https://google.com?q=How+to+backup+CapCut+projects'); // Placeholder, better: invoke 'open_projects_folder'
    // Actually simpler: just open Explorer
    // We haven't implemented 'open_explorer' command yet, so we'll just skip for MVP or ask user to do it manually
  } catch (e) { }
}

async function confirmSwitch() {
  if (!state.pendingSwitchTarget) return;

  const modal = document.getElementById('safety-modal');
  // Show loading state on button
  modal.innerHTML = `
    <div class="modal-content text-center">
      <div class="spinner mb-md" style="margin:0 auto"></div>
      <h3>Switching Version...</h3>
    </div>
  `;

  try {
    const res = await invoke('switch_version', { targetPath: state.pendingSwitchTarget });

    if (res.success) {
      // Reload everything
      await loadInstalledVersions();
      router.navigate('dashboard');
      // Reset Modal
      setTimeout(() => {
        location.reload(); // Simple refresh to clear modal state cleanly
      }, 1000);
    } else {
      alert("Switch failed: " + res.message);
      location.reload();
    }
  } catch (e) {
    alert("Error: " + e);
    location.reload();
  }
}

async function deleteVersion(path) {
  if (!confirm("Are you sure you want to uninstall this version?")) return;

  // Reuse existing protector command or add new 'uninstall_version'
  // For now, we fallback to existing 'delete_versions' approach but scoped to one?
  // Since we didn't implement 'delete_single_version' in backend yet, we will skip implementation for this prototype step.
  alert("Uninstall characteristic pending backend update.");
}

// ============================================
// Library (Archive)
// ============================================
async function loadArchiveVersions() {
  try {
    state.archiveVersions = await invoke('get_archive_versions');
    renderLibrary();
  } catch (e) { }
}

function renderLibrary() {
  const container = document.getElementById('library-grid');
  if (!container) return;

  container.innerHTML = state.archiveVersions.map(v => {
    const isInstalled = state.installedVersions.some(iv => iv.name.includes(v.version));

    return `
      <div class="card" style="height: 100%">
        <div style="display:flex; justify-content:space-between; margin-bottom: 8px">
          <span class="card-title text-accent">${v.persona}</span>
          ${v.risk_level === 'High' ? '<i class="ph-fill ph-warning text-warning"></i>' : ''}
        </div>
        <div class="v-name mb-sm">v${v.version}</div>
        <p class="text-muted text-sm mb-md" style="flex:1">${v.description}</p>

        ${isInstalled ? `
          <button class="btn w-full" disabled>Installed</button>
        ` : `
          <button class="btn btn-primary w-full" onclick="openUrl('${v.download_url}')">
            <i class="ph-bold ph-download-simple"></i> Download
          </button>
        `}
      </div>
    `;
  }).join('');
}

// ============================================
// Cleaner
// ============================================
async function loadCleaner() {
  updateCacheCard();
}

async function runCleaner() {
  const btn = document.querySelector('#view-cleaner button');
  const originalText = btn.innerHTML;
  btn.innerText = "Cleaning...";
  btn.disabled = true;

  try {
    await invoke('clean_cache');
    await loadDashboard(); // refresh stats
    btn.innerHTML = `<i class="ph-bold ph-check"></i> Cleaned!`;
    setTimeout(() => {
      btn.innerHTML = originalText;
      btn.disabled = false;
    }, 2000);
  } catch (e) {
    btn.innerText = "Error";
  }
}

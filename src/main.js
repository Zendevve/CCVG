/**
 * Version Guard - Main Controller
 * macOS 26 Tahoe Design System
 *
 * Security: All DOM manipulation uses safe builder functions (no innerHTML)
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { getVersion } = window.__TAURI__.app;

// ============================================
// Safe DOM Builder Utilities (XSS Prevention)
// ============================================

/**
 * Create an element with attributes, styles, and children (safe - no innerHTML)
 * @param {string} tag - HTML tag name
 * @param {Object} attrs - Attributes (class, id, etc.) and style object
 * @param {...(Node|string)} children - Child elements or text
 * @returns {HTMLElement}
 */
function el(tag, attrs = {}, ...children) {
  const element = document.createElement(tag);

  for (const [key, value] of Object.entries(attrs)) {
    if (key === 'style' && typeof value === 'object') {
      Object.assign(element.style, value);
    } else if (key === 'className') {
      element.className = value;
    } else if (key.startsWith('on') && typeof value === 'function') {
      element.addEventListener(key.slice(2).toLowerCase(), value);
    } else if (key === 'dataset') {
      Object.assign(element.dataset, value);
    } else {
      element.setAttribute(key, value);
    }
  }

  for (const child of children) {
    if (child == null) continue;
    element.append(typeof child === 'string' ? document.createTextNode(child) : child);
  }

  return element;
}

/**
 * Create Phosphor icon element
 * @param {string} iconName - Icon name without 'ph-' prefix (e.g., 'check', 'folder-open')
 * @param {Object} attrs - Additional attributes
 * @returns {HTMLElement}
 */
function icon(iconName, attrs = {}) {
  return el('i', { className: `ph ph-${iconName}`, ...attrs });
}

/**
 * Create text node (escapes HTML automatically)
 * @param {string} text
 * @returns {Text}
 */
function text(str) {
  return document.createTextNode(str);
}

// ============================================
// Modal Confirmation Dialog
// ============================================
const modal = {
  overlay: null,
  resolvePromise: null,

  /**
   * Show a confirmation dialog
   * @param {Object} options
   * @param {string} options.title - Dialog title
   * @param {string} options.message - Dialog message
   * @param {string} options.confirmText - Confirm button text (default: "Confirm")
   * @param {string} options.cancelText - Cancel button text (default: "Cancel")
   * @param {boolean} options.danger - Show danger styling (red button)
   * @param {string} options.iconName - Phosphor icon name (default: "warning-circle")
   * @returns {Promise<boolean>} - Resolves true if confirmed, false if cancelled
   */
  show({ title, message, confirmText = 'Confirm', cancelText = 'Cancel', danger = false, iconName = 'warning-circle' }) {
    return new Promise((resolve) => {
      this.resolvePromise = resolve;
      this.overlay = document.getElementById('modal-overlay');

      const modalIcon = document.getElementById('modal-icon');
      const modalTitle = document.getElementById('modal-title');
      const modalMessage = document.getElementById('modal-message');
      const confirmBtn = document.getElementById('modal-confirm');
      const cancelBtn = document.getElementById('modal-cancel');

      // Update content
      modalIcon.replaceChildren(icon(iconName));
      modalIcon.className = danger ? 'modal-icon danger' : 'modal-icon';
      modalTitle.textContent = title;
      modalMessage.textContent = message;
      confirmBtn.textContent = confirmText;
      cancelBtn.textContent = cancelText;

      // Update danger styling
      if (danger) {
        confirmBtn.classList.add('btn-danger');
      } else {
        confirmBtn.classList.remove('btn-danger');
      }

      // Show modal
      this.overlay.style.display = 'flex';
    });
  },

  hide(result) {
    if (this.overlay) {
      this.overlay.style.display = 'none';
    }
    if (this.resolvePromise) {
      this.resolvePromise(result);
      this.resolvePromise = null;
    }
  }
};

// Modal event handlers
document.getElementById('modal-cancel')?.addEventListener('click', () => modal.hide(false));
document.getElementById('modal-confirm')?.addEventListener('click', () => modal.hide(true));
document.getElementById('modal-overlay')?.addEventListener('click', (e) => {
  if (e.target.id === 'modal-overlay') modal.hide(false);
});


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

// Expose navigation functions for inline handlers
window.goBack = goBack;
window.navigateTo = navigateTo;

// ============================================
// Welcome View Handlers
// ============================================
document.getElementById('btn-start')?.addEventListener('click', () => navigateTo('precheck'));
document.getElementById('btn-switch')?.addEventListener('click', () => navigateTo('switch'));
document.getElementById('btn-legacy')?.addEventListener('click', () => navigateTo('legacy'));
document.getElementById('btn-remove-protection')?.addEventListener('click', removeProtection);

// Load protection status on start
(async function checkProtectionOnLoad() {
  try {
    const status = await invoke('check_protection_status');
    updateStatusCard(status.is_protected);
  } catch (e) {
    console.warn('Could not check protection status:', e);
  }
})();

function updateStatusCard(isProtected) {
  const wrapper = document.getElementById('status-icon-wrapper');
  const icon = document.getElementById('status-icon');
  const title = document.getElementById('status-title');
  const subtitle = document.getElementById('status-subtitle');
  const removeBtn = document.getElementById('btn-remove-protection');

  if (!wrapper || !icon || !title) return;

  if (isProtected) {
    wrapper.className = 'status-icon-wrapper protected';
    icon.className = 'ph ph-shield-check';
    title.innerText = 'Protected';
    subtitle.innerText = 'Your version is locked safe';
    if (removeBtn) removeBtn.style.display = 'inline-flex';
  } else {
    wrapper.className = 'status-icon-wrapper unprotected';
    icon.className = 'ph ph-shield-warning';
    title.innerText = 'Not Protected';
    subtitle.innerText = 'CapCut can update automatically';
    if (removeBtn) removeBtn.style.display = 'none';
  }
}

async function removeProtection() {
  // Show confirmation dialog first
  const confirmed = await modal.show({
    title: 'Remove Protection?',
    message: 'CapCut will be able to auto-update again. You can re-apply protection anytime.',
    confirmText: 'Remove',
    cancelText: 'Keep Protected',
    danger: true,
    iconName: 'shield-slash'
  });

  if (!confirmed) return;

  const btn = document.getElementById('btn-remove-protection');

  // Store original content for reset
  const originalContent = Array.from(btn.childNodes).map(n => n.cloneNode(true));

  btn.disabled = true;
  btn.replaceChildren(icon('circle-notch', { className: 'ph ph-circle-notch spin' }), ' Removing...');

  try {
    const result = await invoke('remove_protection');

    if (result.success) {
      btn.replaceChildren(icon('check'), ' Removed!');
      btn.style.background = 'var(--accent-green)';

      // Update status card
      updateStatusCard(false);

      await sleep(1500);
      btn.style.display = 'none';

      // Reset button state for next time
      btn.disabled = false;
      btn.replaceChildren(...originalContent);
      btn.style.background = '';
    } else {
      throw new Error(result.error);
    }
  } catch (e) {
    btn.replaceChildren(icon('x'), ' Failed');
    btn.style.background = 'var(--accent-red)';
    console.error(e);
    await sleep(2000);
    btn.replaceChildren(icon('shield-slash'), ' Remove Protection');
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

  // Peak-End Rule: Elements
  const heroCheck = document.getElementById('precheck-hero');
  const heroSuccess = document.getElementById('precheck-success');
  const list = document.getElementById('precheck-list');

  // Reset View State
  heroCheck.style.display = 'block';
  heroSuccess.style.display = 'none';
  list.style.display = 'block';
  setStatusIcon(installIcon, 'pending');
  installText.textContent = 'Checking installation...';
  setStatusIcon(processIcon, 'pending');
  processText.textContent = 'Checking processes...';
  nextBtn.disabled = true;

  await sleep(600); // Doherty Threshold: Perceptible delay

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
      // Peak-End Rule: Delightful success state
      await sleep(500);
      heroCheck.style.display = 'none';
      list.style.display = 'none';
      heroSuccess.style.display = 'flex'; // Show success hero

      // Auto-continue or enable button? Let's leave button for control but make it obvious.
      // Actually, standard wizard flow usually auto-advances or enables button.
      // User must click continue to acknowledge functionality.
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

  // Show skeleton loader
  container.replaceChildren(createSkeletonRows(3));

  try {
    const vers = await invoke('scan_versions');
    state.versions = vers;

    if (vers.length === 0) {
      // Empty state with actionable CTA
      const downloadBtn = el('button', { className: 'btn-plain' },
        icon('download-simple'),
        ' Download Legacy Version'
      );
      downloadBtn.addEventListener('click', () => {
        goBack();
        setTimeout(() => navigateTo('legacy'), 100);
      });

      container.replaceChildren(
        el('div', {
          className: 'list-row',
          style: { flexDirection: 'column', textAlign: 'center', padding: '24px' }
        },
          icon('folder-open', { style: { fontSize: '32px', color: 'var(--label-tertiary)', marginBottom: '8px' } }),
          el('span', { className: 'row-title' }, 'No installations found'),
          el('span', { className: 'row-subtitle', style: { marginBottom: '12px' } }, 'Download a legacy version to get started'),
          downloadBtn
        )
      );
      return;
    }

    // Build version list with safe DOM methods
    const fragment = document.createDocumentFragment();
    vers.forEach((v, i) => {
      const row = el('div', {
        className: 'list-row selectable',
        tabindex: '0'
      },
        el('div', { className: 'row-icon bg-accent-indigo' },
          icon('hard-drives')
        ),
        el('div', { className: 'row-content' },
          el('span', { className: 'row-title' }, `CapCut v${v.name}`),
          el('span', { className: 'row-subtitle' }, `${v.size_mb.toFixed(0)} MB`)
        ),
        icon('check', {
          className: 'ph ph-check row-accessory',
          style: { opacity: '0', color: 'var(--accent-blue)', fontSize: '18px' }
        })
      );

      row.addEventListener('click', () => selectVersion(i));
      row.addEventListener('keydown', (e) => handleKey(e, () => selectVersion(i)));
      fragment.append(row);
    });
    container.replaceChildren(fragment);

  } catch (e) {
    container.replaceChildren(
      el('div', { className: 'list-row' },
        el('span', { className: 'row-title', style: { color: 'var(--accent-red)' } }, `Error: ${e}`)
      )
    );
  }
}

window.selectVersion = function (idx) {
  state.selectedVersion = state.versions[idx];
  document.getElementById('btn-continue-version').disabled = false;

  document.querySelectorAll('#version-list .list-row').forEach((el, i) => {
    const check = el.querySelector('.row-accessory');
    if (i === idx) {
      el.classList.add('selected');
      if (check) check.style.opacity = '1';
    } else {
      el.classList.remove('selected');
      if (check) check.style.opacity = '0';
    }
  });
};

// ============================================
// Options View Handlers
// ============================================
document.getElementById('options-back')?.addEventListener('click', goBack);
document.getElementById('btn-apply')?.addEventListener('click', async () => {
  const versionsToDelete = state.versions.filter(v => v.path !== state.selectedVersion.path);

  // If deleting other versions, show confirmation
  if (versionsToDelete.length > 0) {
    const confirmed = await modal.show({
      title: 'Apply Protection?',
      message: `This will permanently delete ${versionsToDelete.length} other version${versionsToDelete.length !== 1 ? 's' : ''} and lock your selected version.`,
      confirmText: 'Apply Protection',
      cancelText: 'Go Back',
      danger: false,
      iconName: 'shield-check'
    });

    if (!confirmed) return;
  }

  runProtectionSequence();
});

// Toggle handlers with keyboard support (Accessibility)
function setupToggle(id, stateKey) {
  const toggle = document.getElementById(id);
  if (!toggle) return;

  const handler = function () {
    state[stateKey] = !state[stateKey];
    this.classList.toggle('on', state[stateKey]);
    this.setAttribute('aria-checked', state[stateKey]);
  };

  toggle.addEventListener('click', handler);
  toggle.addEventListener('keydown', function (e) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handler.call(this);
    }
  });
}

setupToggle('toggle-cache', 'cacheEnabled');
setupToggle('toggle-lock', 'lockEnabled');
setupToggle('toggle-blocker', 'blockerEnabled');

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
  logContainer.replaceChildren();

  const setProgress = (msg, pct) => {
    statusText.textContent = msg;
    progressBar.style.width = `${pct}%`;
  };

  const addLog = (msg, type = 'info') => {
    const iconNames = { ok: 'check', warn: 'warning', info: 'dot' };
    const classes = { ok: 'success', warn: 'warning', info: '' };
    const entry = el('div', { className: `log-entry ${classes[type]}` },
      icon(iconNames[type]),
      el('span', {}, msg)
    );
    logContainer.append(entry);
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
document.getElementById('btn-done')?.addEventListener('click', () => {
  state.history = ['welcome'];
  showView('welcome');
  // Refresh protection status on welcome screen
  (async () => {
    try {
      const status = await invoke('check_protection_status');
      updateStatusCard(status.is_protected);
    } catch (e) { }
  })();
});

// ============================================
// Error View Handlers
// ============================================
document.getElementById('btn-retry')?.addEventListener('click', () => {
  // Start fresh from precheck (where the protection flow begins)
  state.history = ['welcome'];
  navigateTo('precheck');
});
document.getElementById('btn-back-error')?.addEventListener('click', () => {
  // Go back to options so user can try again with different settings
  state.history = ['welcome', 'precheck', 'versions', 'options'];
  showView('options');
});

// ============================================
// Legacy View Handlers
// ============================================
document.getElementById('legacy-back')?.addEventListener('click', goBack);

async function loadArchiveVersions() {
  const container = document.getElementById('legacy-list');
  container.replaceChildren(createSkeletonRows(4));

  try {
    const archives = await invoke('get_archive_versions');

    const fragment = document.createDocumentFragment();
    archives.forEach(v => {
      const riskColor = v.risk_level === 'High' ? 'var(--accent-red)' :
        v.risk_level === 'Medium' ? 'var(--accent-orange)' : 'var(--accent-green)';

      const downloadBtn = el('button', {
        className: 'btn-plain',
        style: { padding: '8px' }
      },
        icon('download-simple', { style: { fontSize: '18px' } })
      );
      downloadBtn.addEventListener('click', () => {
        window.__TAURI__.opener.openUrl(v.download_url);
      });

      fragment.append(
        el('div', { className: 'list-row' },
          el('div', { className: 'row-icon', style: { background: riskColor } },
            icon('package')
          ),
          el('div', { className: 'row-content' },
            el('span', { className: 'row-title' }, `v${v.version} · ${v.persona}`),
            el('span', { className: 'row-subtitle' }, v.description)
          ),
          downloadBtn
        )
      );
    });
    container.replaceChildren(fragment);

  } catch (e) {
    container.replaceChildren(
      el('div', { className: 'list-row' },
        el('span', { className: 'row-title', style: { color: 'var(--accent-red)' } }, `Error: ${e}`)
      )
    );
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
  container.replaceChildren(createSkeletonRows(2));

  try {
    const vers = await invoke('scan_versions');
    state.versions = vers;
    state.switchTarget = null;

    if (vers.length === 0) {
      // Empty state with actionable guidance
      container.replaceChildren(
        el('div', {
          className: 'list-row',
          style: { flexDirection: 'column', textAlign: 'center', padding: '24px' }
        },
          icon('folder-open', { style: { fontSize: '32px', color: 'var(--label-tertiary)', marginBottom: '8px' } }),
          el('span', { className: 'row-title' }, 'No installations found'),
          el('span', { className: 'row-subtitle' }, 'Download a legacy version first')
        )
      );
      document.getElementById('btn-switch-apply').disabled = true;
      return;
    }

    if (vers.length === 1) {
      container.replaceChildren(
        el('div', { className: 'list-row' },
          el('span', { className: 'row-title' }, 'Only one version installed — nothing to switch')
        )
      );
      return;
    }

    // Build switch version list with safe DOM methods
    const fragment = document.createDocumentFragment();
    vers.forEach((v, i) => {
      const row = el('div', {
        className: 'list-row selectable',
        tabindex: '0'
      },
        el('div', { className: 'row-icon bg-accent-purple' },
          icon('hard-drives')
        ),
        el('div', { className: 'row-content' },
          el('span', { className: 'row-title' }, `CapCut v${v.name}`),
          el('span', { className: 'row-subtitle' }, `${v.size_mb.toFixed(0)} MB`)
        ),
        icon('check', {
          className: 'ph ph-check row-accessory',
          style: { opacity: '0', color: 'var(--accent-blue)', fontSize: '18px' }
        })
      );

      row.addEventListener('click', () => selectSwitchVersion(i));
      row.addEventListener('keydown', (e) => handleKey(e, () => selectSwitchVersion(i)));
      fragment.append(row);
    });
    container.replaceChildren(fragment);

  } catch (e) {
    container.replaceChildren(
      el('div', { className: 'list-row' },
        el('span', { className: 'row-title', style: { color: 'var(--accent-red)' } }, `Error: ${e}`)
      )
    );
  }
}

window.selectSwitchVersion = function (idx) {
  state.switchTarget = state.versions[idx];
  document.getElementById('btn-switch-apply').disabled = false;

  document.querySelectorAll('#switch-list .list-row').forEach((el, i) => {
    const check = el.querySelector('.row-accessory');
    if (i === idx) {
      el.classList.add('selected');
      if (check) check.style.opacity = '1';
    } else {
      el.classList.remove('selected');
      if (check) check.style.opacity = '0';
    }
  });
};

async function applySwitch() {
  if (!state.switchTarget) return;

  // Show confirmation dialog
  const confirmed = await modal.show({
    title: 'Switch Version?',
    message: `This will set CapCut v${state.switchTarget.name} as the active version.`,
    confirmText: 'Switch',
    cancelText: 'Cancel',
    danger: false,
    iconName: 'swap'
  });

  if (!confirmed) return;

  const btn = document.getElementById('btn-switch-apply');
  btn.disabled = true;
  btn.replaceChildren(icon('circle-notch', { className: 'ph ph-circle-notch spin' }), ' Switching...');

  try {
    const result = await invoke('switch_version', { targetPath: state.switchTarget.path });

    if (result.success) {
      btn.replaceChildren(icon('check'), ' Switched!');
      btn.style.background = 'var(--accent-green)';
      await sleep(1000);
      state.history = ['welcome'];
      showView('welcome');
      // Reset button for next use
      btn.replaceChildren(icon('swap'), ' Switch Version');
      btn.style.background = '';
      btn.disabled = true; // Reset to disabled state
    } else {
      throw new Error(result.message);
    }
  } catch (e) {
    btn.replaceChildren(icon('x'), ' Failed');
    btn.style.background = 'var(--accent-red)';
    console.error(e);
    await sleep(2000);
    btn.replaceChildren(icon('swap'), ' Switch Version');
    btn.style.background = '';
    btn.disabled = false;
  }
}

// ============================================
// Utilities
// ============================================
function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

// P3: Keyboard Helper
window.handleKey = function (e, action) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    action();
  }
}

/**
 * Create skeleton loading rows (safe DOM version)
 * @param {number} count - Number of skeleton rows
 * @returns {DocumentFragment}
 */
function createSkeletonFragment(count = 3) {
  const fragment = document.createDocumentFragment();
  for (let i = 0; i < count; i++) {
    fragment.append(
      el('div', { className: 'skeleton-row' },
        el('div', { className: 'skeleton-icon' }),
        el('div', { className: 'skeleton-text' },
          el('div', { className: 'skeleton-line medium' }),
          el('div', { className: 'skeleton-line short' })
        )
      )
    );
  }
  return fragment;
}

// Legacy wrapper for skeleton rows (returns fragment for replaceChildren)
function createSkeletonRows(count = 3) {
  return createSkeletonFragment(count);
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

/**
 * System Page - Orchestrator Tab
 */

import API from './api.js';

const SystemOrch = {
    statusPollInterval: null,
    isRunning: false,

    init() {
        this.bindEvents();
        this.loadConfig();
        this.startStatusPolling();
    },

    unload() {
        this.stopStatusPolling();
    },

    bindEvents() {
        const launchBtn = document.getElementById('orch-launch-btn');
        const stopBtn = document.getElementById('orch-stop-btn');
        const saveBtn = document.getElementById('orch-save-config-btn');
        const refreshBtn = document.getElementById('orch-refresh-models');

        if (launchBtn) {
            launchBtn.addEventListener('click', () => this.startService());
        }
        if (stopBtn) {
            stopBtn.addEventListener('click', () => this.stopService());
        }
        if (saveBtn) {
            saveBtn.addEventListener('click', () => this.saveConfig());
        }
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshModels());
        }
    },

    async loadConfig() {
        await this.refreshModels(); // populate dropdown first
        const result = await API.getModelConfig('orchestrator');
        if (result.success && result.data) {
            this.populateForm(result.data);
        }
        await this.updateStatus();
    },

    populateForm(config) {
        const portInput = document.getElementById('orch-port');
        const ctxSizeInput = document.getElementById('orch-ctx-size');
        const outputReserveInput = document.getElementById('orch-output-reserve');
        const nGpuLayersInput = document.getElementById('orch-n-gpu-layers');
        const temperatureInput = document.getElementById('orch-temperature');
        const repeatPenaltyInput = document.getElementById('orch-repeat-penalty');
        const cacheTypeKSelect = document.getElementById('orch-cache-type-k');
        const cacheTypeVSelect = document.getElementById('orch-cache-type-v');
        const flashAttnCheckbox = document.getElementById('orch-flash-attn');
        const cacheRamCheckbox = document.getElementById('orch-cache-ram');
        const modelSelect = document.getElementById('orch-model-select');
        const autoStartCheckbox = document.getElementById('orch-auto-start');

        if (portInput) portInput.value = config.port || '';
        if (ctxSizeInput) ctxSizeInput.value = config.ctx_size || '';
        if (outputReserveInput) outputReserveInput.value = config.output_token_reserve ?? '';
        if (nGpuLayersInput) nGpuLayersInput.value = config.n_gpu_layers ?? '';
        if (temperatureInput) temperatureInput.value = config.temperature ?? '';
        if (repeatPenaltyInput) repeatPenaltyInput.value = config.repeat_penalty ?? '';
        if (cacheTypeKSelect) cacheTypeKSelect.value = config.cache_type_k || '';
        if (cacheTypeVSelect) cacheTypeVSelect.value = config.cache_type_v || '';
        if (flashAttnCheckbox) flashAttnCheckbox.checked = config.flash_attn || false;
        if (cacheRamCheckbox) cacheRamCheckbox.checked = config.cache_ram || false;
        if (autoStartCheckbox) autoStartCheckbox.checked = config.auto_start || false;

        // Select the current model from the dropdown (if already loaded by refreshModels)
        if (modelSelect && config.model_path) {
            const option = modelSelect.querySelector(`option[value="${config.model_path}"]`);
            if (option) {
                option.selected = true;
            } else {
                // Add as a new option if not in list
                const newOption = document.createElement('option');
                newOption.value = config.model_path;
                newOption.textContent = config.model_path.split('/').pop();
                modelSelect.appendChild(newOption);
                newOption.selected = true;
            }
        }
    },

    async saveConfig() {
        const formData = {
            model_type: 'orchestrator',
            port: parseInt(document.getElementById('orch-port')?.value || '0'),
            ctx_size: parseInt(document.getElementById('orch-ctx-size')?.value || '0'),
            output_token_reserve: parseInt(document.getElementById('orch-output-reserve')?.value || '0') || null,
            auto_start: document.getElementById('orch-auto-start')?.checked || false,
            n_gpu_layers: parseInt(document.getElementById('orch-n-gpu-layers')?.value || '0') || null,
            temperature: parseFloat(document.getElementById('orch-temperature')?.value) || null,
            repeat_penalty: parseFloat(document.getElementById('orch-repeat-penalty')?.value) || null,
            cache_type_k: document.getElementById('orch-cache-type-k')?.value || null,
            cache_type_v: document.getElementById('orch-cache-type-v')?.value || null,
            flash_attn: document.getElementById('orch-flash-attn')?.checked || false,
            cache_ram: document.getElementById('orch-cache-ram')?.checked || false,
        };

        const modelSelect = document.getElementById('orch-model-select');
        if (modelSelect?.value) {
            formData.model_path = modelSelect.value;
        }

        const result = await API.updateModelConfig('orchestrator', formData);
        if (result.success) {
            this.showMessage('Configuration saved', 'success');
        } else {
            this.showMessage(`Failed: ${result.error}`, 'error');
        }
    },

    async refreshModels() {
        const modelSelect = document.getElementById('orch-model-select');
        if (!modelSelect) return;

        modelSelect.innerHTML = '<option value="">-- Loading models --</option>';

        const result = await API.listModels('orchestrator');
        if (result.success && result.data) {
            // Group models by repo_name (folder)
            const groups = {};
            result.data.forEach(model => {
                if (!groups[model.repo_name]) {
                    groups[model.repo_name] = [];
                }
                groups[model.repo_name].push(model);
            });

            // Create optgroup for each folder
            Object.keys(groups).sort().forEach(repoName => {
                const group = document.createElement('optgroup');
                group.label = repoName;
                groups[repoName].forEach(model => {
                    const option = document.createElement('option');
                    option.value = model.full_path;
                    option.textContent = model.filename;
                    group.appendChild(option);
                });
                modelSelect.appendChild(group);
            });

            if (modelSelect.options.length === 0) {
                modelSelect.innerHTML = '<option value="">-- No models found --</option>';
            }
        } else {
            modelSelect.innerHTML = '<option value="">-- Error loading --</option>';
        }
    },

    async startService() {
        const launchBtn = document.getElementById('orch-launch-btn');
        const messageEl = document.getElementById('orch-action-message');

        if (launchBtn) launchBtn.disabled = true;
        this.showMessage('Starting orchestrator...', 'info');

        const result = await API.startService('orchestrator');

        if (result.success) {
            this.showMessage('Orchestrator started successfully', 'success');
            await this.updateStatus();
        } else {
            this.showMessage(`Failed: ${result.error}`, 'error');
            if (launchBtn) launchBtn.disabled = false;
        }
    },

    async stopService() {
        const stopBtn = document.getElementById('orch-stop-btn');

        if (stopBtn) stopBtn.disabled = true;
        this.showMessage('Stopping orchestrator...', 'info');

        const result = await API.stopService('orchestrator');

        if (result.success) {
            this.showMessage('Orchestrator stopped', 'success');
            await this.updateStatus();
        } else {
            this.showMessage(`Failed: ${result.error}`, 'error');
            if (stopBtn) stopBtn.disabled = false;
        }
    },

    async updateStatus() {
        const result = await API.getServiceStatus('orchestrator');
        this.isRunning = result.running || false;

        const indicator = document.getElementById('orch-status-indicator');
        const statusText = document.getElementById('orch-status-text');
        const launchBtn = document.getElementById('orch-launch-btn');
        const stopBtn = document.getElementById('orch-stop-btn');

        if (indicator) {
            indicator.classList.toggle('status-running', this.isRunning);
            indicator.classList.toggle('status-stopped', !this.isRunning);
        }
        if (statusText) {
            statusText.textContent = this.isRunning ? 'Running' : 'Stopped';
        }
        if (launchBtn) launchBtn.disabled = this.isRunning;
        if (stopBtn) stopBtn.disabled = !this.isRunning;
    },

    startStatusPolling() {
        this.statusPollInterval = setInterval(() => this.updateStatus(), 5000);
    },

    stopStatusPolling() {
        if (this.statusPollInterval) {
            clearInterval(this.statusPollInterval);
            this.statusPollInterval = null;
        }
    },

    showMessage(message, type = 'info') {
        const messageEl = document.getElementById('orch-action-message');
        if (!messageEl) return;

        messageEl.textContent = message;
        messageEl.className = `message message-${type}`;
        messageEl.classList.remove('hidden');

        // Auto-hide after 5 seconds
        setTimeout(() => {
            messageEl.classList.add('hidden');
        }, 5000);
    },
};

export default SystemOrch;

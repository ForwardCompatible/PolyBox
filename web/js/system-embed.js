/**
 * System Page - Embedding Tab
 */

import API from './api.js';

const SystemEmbed = {
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
        const startBtn = document.getElementById('emb-start-btn');
        const stopBtn = document.getElementById('emb-stop-btn');
        const saveBtn = document.getElementById('emb-save-btn');
        const refreshBtn = document.getElementById('emb-refresh-models');

        if (startBtn) {
            startBtn.addEventListener('click', () => this.startService());
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
        const [modelResult, appResult] = await Promise.all([
            API.getModelConfig('embedding'),
            API.getAppSettings(),
        ]);
        if (modelResult.success && modelResult.data) {
            this.populateForm(modelResult.data, appResult.data);
        }
        await this.updateStatus();
    },

    populateForm(config, appSettings = {}) {
        const portInput = document.getElementById('emb-port');
        const ctxSizeInput = document.getElementById('emb-ctx-size');
        const dimInput = document.getElementById('emb-dim');
        const modelSelect = document.getElementById('emb-model-select');
        const autoStartCheckbox = document.getElementById('emb-auto-start');

        if (portInput) portInput.value = config.port || '';
        if (ctxSizeInput) ctxSizeInput.value = config.embedding_ctx_size || '';
        if (dimInput) dimInput.value = appSettings.embeddings_dim || '';
        if (autoStartCheckbox) autoStartCheckbox.checked = config.auto_start || false;

        // Select the current model if set
        if (modelSelect && config.model_path) {
            const option = modelSelect.querySelector(`option[value="${config.model_path}"]`);
            if (option) {
                option.selected = true;
            } else if (config.model_path) {
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
        // Save embedding config (model launch settings) — dim is NOT included here
        const formData = {
            model_type: 'embedding',
            port: parseInt(document.getElementById('emb-port')?.value || '0'),
            embedding_ctx_size: parseInt(document.getElementById('emb-ctx-size')?.value || '0'),
            auto_start: document.getElementById('emb-auto-start')?.checked || false,
        };

        const modelSelect = document.getElementById('emb-model-select');
        if (modelSelect?.value) {
            formData.model_path = modelSelect.value;
        }

        const result = await API.updateModelConfig('embedding', formData);
        if (!result.success) {
            this.showMessage(`Failed: ${result.error}`, 'error');
            return;
        }

        // Save dim to app settings (request-time parameter, not a launch flag)
        const dim = parseInt(document.getElementById('emb-dim')?.value || '0') || null;
        const appResult = await API.updateAppSettings({ embeddings_dim: dim });
        if (appResult.success) {
            this.showMessage('Configuration saved', 'success');
        } else {
            this.showMessage(`Config saved but dim failed: ${appResult.error}`, 'error');
        }
    },

    async refreshModels() {
        const modelSelect = document.getElementById('emb-model-select');
        if (!modelSelect) return;

        modelSelect.innerHTML = '<option value="">-- Loading models --</option>';

        const result = await API.listModels('embedding');
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
        const startBtn = document.getElementById('emb-start-btn');
        const messageEl = document.getElementById('emb-action-message');

        if (startBtn) startBtn.disabled = true;
        this.showMessage('Starting embedding service...', 'info');

        const result = await API.startService('embedding');

        if (result.success) {
            this.showMessage('Embedding service started', 'success');
            await this.updateStatus();
        } else {
            this.showMessage(`Failed: ${result.error}`, 'error');
            if (startBtn) startBtn.disabled = false;
        }
    },

    async stopService() {
        const stopBtn = document.getElementById('emb-stop-btn');

        if (stopBtn) stopBtn.disabled = true;
        this.showMessage('Stopping embedding service...', 'info');

        const result = await API.stopService('embedding');

        if (result.success) {
            this.showMessage('Embedding service stopped', 'success');
            await this.updateStatus();
        } else {
            this.showMessage(`Failed: ${result.error}`, 'error');
            if (stopBtn) stopBtn.disabled = false;
        }
    },

    async updateStatus() {
        const result = await API.getServiceStatus('embedding');
        this.isRunning = result.running || false;

        const indicator = document.getElementById('emb-status-indicator');
        const statusText = document.getElementById('emb-status-text');
        const startBtn = document.getElementById('emb-start-btn');
        const stopBtn = document.getElementById('emb-stop-btn');

        if (indicator) {
            indicator.classList.toggle('status-running', this.isRunning);
            indicator.classList.toggle('status-stopped', !this.isRunning);
        }
        if (statusText) {
            statusText.textContent = this.isRunning ? 'Running' : 'Stopped';
        }
        if (startBtn) startBtn.disabled = this.isRunning;
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
        const messageEl = document.getElementById('emb-action-message');
        if (!messageEl) return;

        messageEl.textContent = message;
        messageEl.className = `message message-${type}`;
        messageEl.classList.remove('hidden');

        setTimeout(() => {
            messageEl.classList.add('hidden');
        }, 5000);
    },
};

export default SystemEmbed;

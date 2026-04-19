/**
 * Settings Page
 * Handles loading and saving app_settings with proper field name mapping
 * Frontend uses kebab-case (agent-name), Backend uses snake_case (agent_name)
 */

import API from './api.js';

const Settings = {
    // Mapping from frontend form IDs (kebab-case) to backend field names (snake_case)
    fieldMap: {
        'agent-name': 'agent_name',
        'user-name': 'user_name',
        'timezone': 'user_timezone',
        'max-iterations': 'max_iterations',
        'thinking-open-tag': 'thinking_open_tag',
        'thinking-close-tag': 'thinking_close_tag',
        'show-reasoning': 'show_reasoning',
        'reasoning-collapsed': 'reasoning_collapsed_default',
        'store-context-debug': 'store_context_debug',
        'embeddings-enabled': 'embeddings_enabled',
    },

    // Reverse mapping for populating form
    reverseFieldMap: {},

    init() {
        // Build reverse map
        for (const [formId, backendField] of Object.entries(this.fieldMap)) {
            this.reverseFieldMap[backendField] = formId;
        }

        this.bindEvents();
        this.loadSettings();
    },

    unload() {
        // Cleanup if needed
    },

    bindEvents() {
        const saveBtn = document.getElementById('save-settings-btn');
        if (saveBtn) {
            saveBtn.addEventListener('click', () => this.saveSettings());
        }
    },

    async loadSettings() {
        const result = await API.getAppSettings();
        if (result.success && result.data) {
            this.populateForm(result.data);
        } else {
            this.showMessage('Failed to load settings', 'error');
        }
    },

    populateForm(settings) {
        for (const [backendField, value] of Object.entries(settings)) {
            const formId = this.reverseFieldMap[backendField];
            if (!formId) continue;

            const element = document.getElementById(formId);
            if (!element) continue;

            if (element.type === 'checkbox') {
                element.checked = Boolean(value);
            } else {
                element.value = value ?? '';
            }
        }
    },

    async saveSettings() {
        const formData = {};
        for (const [formId, backendField] of Object.entries(this.fieldMap)) {
            const element = document.getElementById(formId);
            if (!element) continue;
            if (element.type === 'checkbox') {
                formData[backendField] = element.checked;
            } else {
                formData[backendField] = element.value;
            }
        }
        if (formData.max_iterations) {
            formData.max_iterations = parseInt(formData.max_iterations, 10);
        }
        formData.id = 1; // Required for the update

        const result = await API.updateAppSettings(formData);
        if (result.success) {
            this.showMessage('Settings saved', 'success');
        } else {
            this.showMessage(`Failed: ${result.error}`, 'error');
        }
    },

    showMessage(message, type = 'info') {
        const messageEl = document.getElementById('settings-action-message');
        if (!messageEl) return;

        messageEl.textContent = message;
        messageEl.className = `message message-${type}`;
        messageEl.classList.remove('hidden');

        setTimeout(() => {
            messageEl.classList.add('hidden');
        }, 5000);
    },
};

export default Settings;

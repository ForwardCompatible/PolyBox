/**
 * Database Page - Backup and Restore functionality
 */

import API from './api.js';

const Database = {
    init() {
        this.bindEvents();
        this.loadBackups();
    },

    unload() {
        // Cleanup if needed
    },

    bindEvents() {
        const backupBtn = document.getElementById('database-backup-btn');
        const restoreSelect = document.getElementById('database-restore-select');
        const restoreBtn = document.getElementById('database-restore-btn');

        if (backupBtn) {
            backupBtn.addEventListener('click', () => this.createBackup());
        }
        if (restoreSelect) {
            restoreSelect.addEventListener('change', () => this.onBackupSelected());
        }
        if (restoreBtn) {
            restoreBtn.addEventListener('click', () => this.restoreBackup());
        }
    },

    async loadBackups() {
        const result = await API.listBackups();
        const select = document.getElementById('database-restore-select');
        const restoreBtn = document.getElementById('database-restore-btn');

        if (!select) return;

        // Clear existing options
        select.innerHTML = '<option value="">-- Select backup --</option>';

        if (result.success && result.data) {
            result.data.forEach(backup => {
                const option = document.createElement('option');
                option.value = backup;
                option.textContent = backup;
                select.appendChild(option);
            });
        }

        this.onBackupSelected();
    },

    onBackupSelected() {
        const select = document.getElementById('database-restore-select');
        const restoreBtn = document.getElementById('database-restore-btn');

        if (select && restoreBtn) {
            restoreBtn.disabled = !select.value;
        }
    },

    async createBackup() {
        const backupBtn = document.getElementById('database-backup-btn');
        const messageEl = document.getElementById('database-action-message');

        if (backupBtn) backupBtn.disabled = true;
        this.showMessage('Creating backup...', 'info');

        const result = await API.createBackup();

        if (result.success) {
            this.showMessage(`Backup created: ${result.data.path}`, 'success');
            await this.loadBackups();
        } else {
            this.showMessage(`Backup failed: ${result.error}`, 'error');
        }

        if (backupBtn) backupBtn.disabled = false;
    },

    async restoreBackup() {
        const select = document.getElementById('database-restore-select');
        const restoreBtn = document.getElementById('database-restore-btn');

        if (!select?.value) {
            this.showMessage('Please select a backup to restore', 'warning');
            return;
        }

        const confirmed = confirm(`Restore from backup "${select.value}"? This will restart the system.`);
        if (!confirmed) return;

        if (restoreBtn) restoreBtn.disabled = true;
        this.showMessage('Restoring...', 'info');

        const result = await API.restoreBackup(select.value);

        if (result.success) {
            this.showMessage('Restore successful. Restarting...', 'success');
        } else {
            this.showMessage(`Restore failed: ${result.error}`, 'error');
            if (restoreBtn) restoreBtn.disabled = false;
        }
    },

    showMessage(message, type = 'info') {
        const messageEl = document.getElementById('database-action-message');
        if (!messageEl) return;

        messageEl.textContent = message;
        messageEl.className = `message message-${type}`;
        messageEl.classList.remove('hidden');

        if (type !== 'warning') {
            setTimeout(() => {
                messageEl.classList.add('hidden');
            }, 5000);
        }
    },
};

export default Database;

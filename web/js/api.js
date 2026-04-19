/**
 * API Client for PolyBox REST endpoints
 */

const API = {
    baseUrl: '',

    async request(method, path, body = null) {
        const options = {
            method,
            headers: {
                'Content-Type': 'application/json',
            },
        };

        if (body) {
            options.body = JSON.stringify(body);
        }

        try {
            const response = await fetch(this.baseUrl + path, options);
            const data = await response.json();
            return data;
        } catch (error) {
            return {
                success: false,
                error: error.message,
            };
        }
    },

    // Hardware stats
    async getHardware() {
        return this.request('GET', '/api/hardware');
    },

    // Service status
    async getServiceStatus(serviceType) {
        return this.request('GET', `/api/services/${serviceType}/status`);
    },

    async startService(serviceType) {
        return this.request('POST', `/api/services/${serviceType}/start`);
    },

    async stopService(serviceType) {
        return this.request('POST', `/api/services/${serviceType}/stop`);
    },

    // App settings
    async getAppSettings() {
        return this.request('GET', '/api/app-settings');
    },

    async updateAppSettings(settings) {
        return this.request('PUT', '/api/app-settings', settings);
    },

    // Model config
    async getModelConfig(modelType) {
        return this.request('GET', `/api/model-configs/${modelType}`);
    },

    async updateModelConfig(modelType, config) {
        return this.request('PUT', `/api/model-configs/${modelType}`, config);
    },

    // Action registry
    async getActionRegistry() {
        return this.request('GET', '/api/action-registry');
    },

    async listModels(modelType) {
        return this.request('GET', `/api/models/${modelType}`);
    },

    // Chat history
    async getChatHistory(sessionId, limit = 50) {
        return this.request('GET', `/api/chat/${sessionId}?limit=${limit}`);
    },

    // Database
    async createBackup() {
        return this.request('POST', '/api/database/backup');
    },

    async listBackups() {
        return this.request('GET', '/api/database/backups');
    },

    async restoreBackup(backupName) {
        return this.request('POST', '/api/database/restore', { backup_name: backupName });
    },
};

export default API;

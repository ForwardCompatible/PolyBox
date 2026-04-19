/**
 * Footer module — Hardware stats polling
 * Polls /api/hardware every second for CPU/RAM/VRAM display
 */

import API from './api.js';

const Footer = {
    pollInterval: null,

    init() {
        this.startPolling();
    },

    unload() {
        this.stopPolling();
    },

    startPolling() {
        this.pollInterval = setInterval(() => this.fetchStats(), 1000);
        this.fetchStats(); // fetch immediately
    },

    stopPolling() {
        if (this.pollInterval) {
            clearInterval(this.pollInterval);
            this.pollInterval = null;
        }
    },

    async fetchStats() {
        const [hwResult, orchResult, embedResult] = await Promise.all([
            API.getHardware(),
            API.getServiceStatus('orchestrator'),
            API.getServiceStatus('embedding'),
        ]);

        if (hwResult.success && hwResult.data) {
            this.updateDisplay(hwResult.data);
        }

        this.updateServiceStatus({
            orchestrator: orchResult?.running ?? false,
            embedding: embedResult?.running ?? false,
        });
    },

    updateDisplay(data) {
        const cpuEl = document.getElementById('cpu-value');
        const ramEl = document.getElementById('ram-value');
        const vramEl = document.getElementById('vram-value');

        if (cpuEl) cpuEl.textContent = `${data.cpu_percent.toFixed(1)}%`;
        if (ramEl) ramEl.textContent = `${data.ram_used_gb.toFixed(1)} / ${data.ram_total_gb.toFixed(1)} GB`;

        if (vramEl) {
            if (data.vram_total_gb !== null && data.vram_total_gb !== undefined) {
                vramEl.textContent = `${data.vram_used_gb.toFixed(1)} / ${data.vram_total_gb.toFixed(1)} GB`;
            } else {
                vramEl.textContent = '-- / -- GB';
            }
        }
    },

    updateServiceStatus(data) {
        const orchIndicator = document.getElementById('orch-status');
        const embedIndicator = document.getElementById('embed-status');

        if (orchIndicator) {
            orchIndicator.classList.toggle('running', data.orchestrator);
            orchIndicator.classList.toggle('error', false);
        }
        if (embedIndicator) {
            embedIndicator.classList.toggle('running', data.embedding);
            embedIndicator.classList.toggle('error', false);
        }
    },
};

export default Footer;

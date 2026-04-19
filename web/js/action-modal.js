/**
 * Action Tag Modal
 * Displays available action tags and allows inserting them into chat
 */

import API from './api.js';

const ActionModal = {
    modal: null,
    actionList: null,
    actions: [],

    init() {
        this.modal = document.getElementById('action-modal');
        this.actionList = document.getElementById('action-list');

        if (!this.modal) {
            console.warn('Action modal not found');
            return;
        }

        this.bindEvents();
        this.loadActions();
    },

    bindEvents() {
        // Close button
        const closeBtn = this.modal.querySelector('.modal-close');
        if (closeBtn) {
            closeBtn.addEventListener('click', () => this.close());
        }

        // Click outside to close
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.close();
            }
        });

        // Action button in chat input bar
        const actionBtn = document.querySelector('.chat-action-btn');
        if (actionBtn) {
            actionBtn.addEventListener('click', () => this.open());
        }
    },

    async loadActions() {
        const result = await API.getActionRegistry();
        if (result.success && result.data) {
            this.actions = result.data;
            this.renderActions();
        }
    },

    renderActions() {
        if (!this.actionList) return;

        this.actionList.innerHTML = '';

        this.actions.forEach(action => {
            const row = document.createElement('div');
            row.className = 'action-row';

            const tagSpan = document.createElement('span');
            tagSpan.className = 'action-tag';
            tagSpan.textContent = `[${action.tag}]`;

            const descSpan = document.createElement('span');
            descSpan.className = 'action-description';
            descSpan.textContent = action.description;

            row.appendChild(tagSpan);
            row.appendChild(descSpan);

            row.addEventListener('click', () => {
                this.selectAction(action);
            });

            this.actionList.appendChild(row);
        });
    },

    selectAction(action) {
        // Dispatch custom event with action data
        const event = new CustomEvent('action-select', {
            detail: { action },
        });
        document.dispatchEvent(event);

        this.close();
    },

    open() {
        if (this.modal) {
            this.modal.classList.remove('hidden');
            this.loadActions(); // Refresh actions each time
        }
    },

    close() {
        if (this.modal) {
            this.modal.classList.add('hidden');
        }
    },
};

export default ActionModal;

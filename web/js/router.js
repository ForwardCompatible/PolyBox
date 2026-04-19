/**
 * Page Router for PolyBox
 * Loads page fragments and initializes appropriate JS modules
 */

const Router = {
    currentPage: null,
    pageModules: {},

    async init(pageModules) {
        this.pageModules = pageModules;

        // Set up navigation listeners
        document.querySelectorAll('.nav-item[data-page]').forEach(item => {
            item.addEventListener('click', () => {
                const page = item.dataset.page;
                this.navigateTo(page);
            });
        });

        // Load initial page
        const hash = window.location.hash.slice(1) || 'chat';
        this.navigateTo(hash, true);
    },

    async navigateTo(page, suppressHistory = false) {
        if (this.currentPage === page) return;

        // Update sidebar active state
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.toggle('active', item.dataset.page === page);
        });

        // Unload current page
        if (this.currentPage && this.pageModules[this.currentPage]?.unload) {
            await this.pageModules[this.currentPage].unload();
        }

        // Load new page
        const container = document.getElementById('content-container');
        if (!container) {
            console.error('Content container not found');
            return;
        }

        try {
            const response = await fetch(`/pages/${page}.html`);
            if (!response.ok) {
                throw new Error(`Failed to load ${page}.html`);
            }
            const html = await response.text();
            container.innerHTML = html;

            this.currentPage = page;

            // Initialize new page module
            if (this.pageModules[page]?.init) {
                this.pageModules[page].init();
            }

            // Update URL hash
            if (!suppressHistory) {
                window.location.hash = page;
            }
        } catch (error) {
            console.error('Navigation error:', error);
            container.innerHTML = `<div class="error-message">Failed to load page: ${error.message}</div>`;
        }
    },

    getCurrentPage() {
        return this.currentPage;
    },
};

export default Router;

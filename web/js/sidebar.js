/**
 * Sidebar navigation controller
 */

const Sidebar = {
    collapsed: false,

    init() {
        const toggle = document.getElementById('sidebar-toggle');
        if (toggle) {
            toggle.addEventListener('click', () => this.toggle());
        }

        // Handle responsive collapse
        if (window.innerWidth < 768) {
            this.collapse();
        }
    },

    toggle() {
        if (this.collapsed) {
            this.expand();
        } else {
            this.collapse();
        }
    },

    collapse() {
        document.getElementById('sidebar')?.classList.add('collapsed');
        document.querySelector('.toggle-icon').textContent = '▶';
        this.collapsed = true;
    },

    expand() {
        document.getElementById('sidebar')?.classList.remove('collapsed');
        document.querySelector('.toggle-icon').textContent = '◀';
        this.collapsed = false;
    },
};

export default Sidebar;

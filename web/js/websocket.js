/**
 * WebSocket client for PolyBox
 * Implements exponential backoff reconnection per SPEC.md
 */

const WS = {
    socket: null,
    url: '',
    handlers: {},
    reconnectAttempts: 0,
    maxReconnectDelay: 30000, // 30s max
    baseDelay: 1000, // 1s initial
    reconnectTimer: null,
    shouldReconnect: true,

    connect(url) {
        // Skip if already connected to the same URL
        if (this.socket && this.socket.readyState === WebSocket.OPEN && this.url === url) {
            return;
        }
        this.url = url;
        this.shouldReconnect = true;
        this.doConnect();
    },

    doConnect() {
        if (this.socket) {
            this.socket.close();
        }

        this.socket = new WebSocket(this.url);

        this.socket.onopen = () => {
            console.log('WebSocket connected');
            this.reconnectAttempts = 0;
            this.emit('connected');
        };

        this.socket.onclose = () => {
            console.log('WebSocket closed');
            this.emit('disconnected');
            this.scheduleReconnect();
        };

        this.socket.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.emit('error', error);
        };

        this.socket.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                this.handleMessage(data);
            } catch (e) {
                console.error('Failed to parse WebSocket message:', e);
            }
        };
    },

    scheduleReconnect() {
        if (!this.shouldReconnect) return;

        // Calculate delay with ±20% jitter
        const jitter = 0.8 + Math.random() * 0.4; // 0.8 to 1.2
        const delay = Math.min(
            this.baseDelay * Math.pow(2, this.reconnectAttempts) * jitter,
            this.maxReconnectDelay
        );

        console.log(`Scheduling reconnect in ${Math.round(delay)}ms`);
        this.reconnectTimer = setTimeout(() => {
            this.reconnectAttempts++;
            this.doConnect();
        }, delay);
    },

    disconnect() {
        this.shouldReconnect = false;
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }
    },

    send(data) {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
            this.socket.send(typeof data === 'string' ? data : JSON.stringify(data));
            return true;
        }
        return false;
    },

    sendMessage(content) {
        return this.send({ type: 'message', content });
    },

    sendStop() {
        return this.send({ type: 'stop' });
    },

    sendGetHistory(sessionId, limit) {
        return this.send({ type: 'get_history', session_id: sessionId, limit });
    },

    on(event, handler) {
        if (!this.handlers[event]) {
            this.handlers[event] = [];
        }
        this.handlers[event].push(handler);
    },

    off(event, handler) {
        if (!this.handlers[event]) return;
        this.handlers[event] = this.handlers[event].filter(h => h !== handler);
    },

    emit(event, ...args) {
        if (!this.handlers[event]) return;
        this.handlers[event].forEach(handler => handler(...args));
    },

    handleMessage(data) {
        const { type } = data;

        // Emit specific event (footer.js handles DOM updates for hardware_monitor and service_status)
        this.emit(type, data);

        // Also emit 'message' for all messages
        this.emit('message', data);
    },
};

export default WS;

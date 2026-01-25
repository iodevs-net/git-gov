"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Transport = void 0;
const net = require("net");
const SOCKET_PATH = '/tmp/cliff-watch-sensor.sock';
class Transport {
    constructor() {
        this.client = null;
        this.buffer = [];
        this.isConnecting = false;
        this.retryDelay = 1000;
        this.connect();
    }
    connect() {
        if (this.isConnecting || this.client)
            return;
        this.isConnecting = true;
        const socket = net.createConnection({ path: SOCKET_PATH }, () => {
            console.log('Connected to Cliff-Watch Daemon');
            this.client = socket;
            this.isConnecting = false;
            this.retryDelay = 1000; // Reset backoff
            this.flushBuffer();
        });
        socket.on('error', (err) => {
            // Silent error handling
            this.client = null;
            this.isConnecting = false;
            this.scheduleReconnect();
        });
        socket.on('close', () => {
            this.client = null;
            this.isConnecting = false;
            this.scheduleReconnect();
        });
        // Ensure the socket doesn't keep the event loop alive if it's the only thing left
        socket.unref();
    }
    scheduleReconnect() {
        // Exponential backoff with max cap
        const delay = this.retryDelay;
        this.retryDelay = Math.min(this.retryDelay * 2, 30000); // Cap at 30s
        setTimeout(() => {
            this.connect();
        }, delay);
    }
    send(event) {
        if (this.client && !this.client.destroyed) {
            try {
                // Protocol expects one JSON per line
                const success = this.client.write(JSON.stringify(event) + '\n');
                if (!success) {
                    // Backpressure handling?
                    // For now, we trust Node to buffer internally, or if it fails, error event triggers.
                }
            }
            catch (e) {
                this.bufferEvent(event);
            }
        }
        else {
            this.bufferEvent(event);
        }
    }
    bufferEvent(event) {
        if (this.buffer.length < 100) { // Max 100 events buffer
            this.buffer.push(event);
        }
    }
    flushBuffer() {
        while (this.buffer.length > 0 && this.client) {
            const event = this.buffer.shift();
            if (event) {
                this.send(event);
            }
        }
    }
    dispose() {
        if (this.client) {
            this.client.end();
            this.client = null;
        }
    }
}
exports.Transport = Transport;
//# sourceMappingURL=transport.js.map
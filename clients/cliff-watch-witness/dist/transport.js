"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Transport = void 0;
const net = require("net");
const child_process = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");
const SOCKET_PATH = '/tmp/cliff-watch-sensor.sock';
class Transport {
    constructor() {
        this.client = null;
        this.buffer = [];
        this.isConnecting = false;
        this.retryDelay = 1000;
        this.daemonProcess = null;
        this.ensureDaemonRunning().then(() => {
            this.connect();
        });
    }
    async ensureDaemonRunning() {
        // Si el socket ya existe, el daemon probablemente está corriendo
        if (fs.existsSync(SOCKET_PATH)) {
            return;
        }
        const platform = os.platform();
        const arch = os.arch();
        // Buscamos el binario embebido basado en SO-Arch
        // Por ahora soportamos linux-x64 como el binario que acabamos de compilar
        let binName = `cliff-watch-daemon-${platform}-${arch}`;
        if (platform === 'win32')
            binName += '.exe';
        // La ruta del binario en el paquete VSIX instalado
        const binPath = path.join(__dirname, '..', 'bin', binName);
        if (fs.existsSync(binPath)) {
            console.log(`Starting embedded Cliff-Watch Daemon: ${binPath}`);
            try {
                this.daemonProcess = child_process.spawn(binPath, [], {
                    detached: true,
                    stdio: 'ignore'
                });
                this.daemonProcess.unref();
                // Damos un momento para que el socket se cree
                await new Promise(resolve => setTimeout(resolve, 500));
            }
            catch (e) {
                console.error('Failed to start embedded daemon:', e);
            }
        }
        else {
            console.warn(`No embedded daemon found at ${binPath}. Expecting global cliff-watch-daemon.`);
        }
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
        // No matamos el daemonProcess para que siga "vigilando" incluso si se cierra VSCode
        // Cliff-Watch es soberano y persistente por diseño.
    }
}
exports.Transport = Transport;
//# sourceMappingURL=transport.js.map
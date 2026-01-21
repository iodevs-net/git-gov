import * as net from 'net';
import { SensorEvent } from './types';

const SOCKET_PATH = '/tmp/git-gov-sensor.sock';

export class Transport {
    private client: net.Socket | null = null;
    private buffer: SensorEvent[] = [];
    private isConnecting: boolean = false;
    private retryDelay: number = 1000;

    constructor() {
        this.connect();
    }

    public connect() {
        if (this.isConnecting || this.client) return;
        this.isConnecting = true;

        const socket = net.createConnection({ path: SOCKET_PATH }, () => {
            console.log('Connected to Git-Gov Daemon');
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

    private scheduleReconnect() {
        // Exponential backoff with max cap
        const delay = this.retryDelay;
        this.retryDelay = Math.min(this.retryDelay * 2, 30000); // Cap at 30s

        setTimeout(() => {
            this.connect();
        }, delay);
    }

    public send(event: SensorEvent) {
        if (this.client && !this.client.destroyed) {
            try {
                // Protocol expects one JSON per line
                const success = this.client.write(JSON.stringify(event) + '\n');
                if (!success) {
                    // Backpressure handling?
                    // For now, we trust Node to buffer internally, or if it fails, error event triggers.
                }
            } catch (e) {
                this.bufferEvent(event);
            }
        } else {
            this.bufferEvent(event);
        }
    }

    private bufferEvent(event: SensorEvent) {
        if (this.buffer.length < 100) { // Max 100 events buffer
            this.buffer.push(event);
        }
    }

    private flushBuffer() {
        while (this.buffer.length > 0 && this.client) {
            const event = this.buffer.shift();
            if (event) {
                this.send(event);
            }
        }
    }

    public dispose() {
        if (this.client) {
            this.client.end();
            this.client = null;
        }
    }
}

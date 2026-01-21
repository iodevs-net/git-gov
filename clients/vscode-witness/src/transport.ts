import * as net from 'net';
import { SensorEvent } from './types';

export const DEFAULT_SOCKET_PATH = '/tmp/git-gov-sensor.sock';
const MAX_BUFFER_SIZE = 100;
const INITIAL_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;

export class Transport {
    private socket: net.Socket | null = null;
    private buffer: SensorEvent[] = [];
    private reconnectDelay = INITIAL_RECONNECT_DELAY;
    private isConnecting = false;
    private socketPath: string;

    constructor(socketPath: string = DEFAULT_SOCKET_PATH) {
        this.socketPath = socketPath;
        this.connect();
    }

    private connect() {
        if (this.isConnecting) {return;}
        this.isConnecting = true;

        this.socket = net.createConnection({ path: this.socketPath }, () => {
            console.log('Connected to git-gov-sensor socket');
            this.isConnecting = false;
            this.reconnectDelay = INITIAL_RECONNECT_DELAY;
            this.flushBuffer();
        });

        this.socket.on('error', (err) => {
            // Silent failure
            this.isConnecting = false;
            this.scheduleReconnect();
        });

        this.socket.on('close', () => {
            this.socket = null;
            this.isConnecting = false;
            this.scheduleReconnect();
        });
    }

    private scheduleReconnect() {
        if (this.isConnecting) {return;} // Already trying to connect

        setTimeout(() => {
            this.connect();
        }, this.reconnectDelay);

        this.reconnectDelay = Math.min(this.reconnectDelay * 2, MAX_RECONNECT_DELAY);
    }

    public send(event: SensorEvent) {
        if (this.socket && !this.socket.connecting && !this.socket.destroyed) {
            try {
                const json = JSON.stringify(event);
                this.socket.write(json + '\n');
            } catch (e) {
                this.bufferEvent(event);
            }
        } else {
            this.bufferEvent(event);
        }
    }

    private bufferEvent(event: SensorEvent) {
        if (this.buffer.length >= MAX_BUFFER_SIZE) {
            this.buffer.shift(); // Drop oldest
        }
        this.buffer.push(event);
    }

    private flushBuffer() {
        if (!this.socket) {return;}

        while (this.buffer.length > 0) {
            const event = this.buffer.shift();
            if (event) {
                this.send(event);
            }
        }
    }

    public dispose() {
        if (this.socket) {
            this.socket.end();
            this.socket.destroy();
            this.socket = null;
        }
    }
}

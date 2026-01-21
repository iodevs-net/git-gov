import { describe, test, expect, beforeAll, afterAll, vi } from 'vitest';
import * as net from 'net';
import * as fs from 'fs';
import { Transport } from '../src/transport';
import { SensorEvent } from '../src/types';
import * as path from 'path';

const TEST_SOCKET_PATH = path.join('/tmp', `git-gov-sensor-test-${Date.now()}.sock`);

describe('Transport', () => {
    let server: net.Server;
    let receivedData: string[] = [];

    beforeAll(async () => {
        // Clean up if exists
        if (fs.existsSync(TEST_SOCKET_PATH)) {
            try {
                fs.unlinkSync(TEST_SOCKET_PATH);
            } catch (e) {}
        }

        server = net.createServer((socket) => {
            socket.on('data', (data) => {
                const lines = data.toString().split('\n').filter(line => line.trim() !== '');
                receivedData.push(...lines);
            });
        });

        await new Promise<void>((resolve) => {
            server.listen(TEST_SOCKET_PATH, () => {
                resolve();
            });
        });
    });

    afterAll(() => {
        server.close();
        if (fs.existsSync(TEST_SOCKET_PATH)) {
             try {
                fs.unlinkSync(TEST_SOCKET_PATH);
            } catch (e) {}
        }
    });

    test('should connect and send events', async () => {
        const transport = new Transport(TEST_SOCKET_PATH);

        // Wait for connection
        await new Promise((resolve) => setTimeout(resolve, 500));

        const event: SensorEvent = {
            type: 'heartbeat',
            timestamp_ms: 123456789
        };

        transport.send(event);

        // Wait for data
        await new Promise((resolve) => setTimeout(resolve, 500));

        expect(receivedData.length).toBeGreaterThan(0);
        const received = JSON.parse(receivedData[0]);
        expect(received).toEqual(event);

        transport.dispose();
    });

    test('should buffer events when disconnected', async () => {
        // Stop server
        server.close();
        if (fs.existsSync(TEST_SOCKET_PATH)) {
            fs.unlinkSync(TEST_SOCKET_PATH);
        }

        const transport = new Transport(TEST_SOCKET_PATH);

        // Wait a bit (it will fail to connect)
        await new Promise((resolve) => setTimeout(resolve, 500));

        const event1: SensorEvent = { type: 'focus_lost', timestamp_ms: 1000 };
        const event2: SensorEvent = { type: 'focus_gained', file_path: 'test', timestamp_ms: 2000 };

        transport.send(event1);
        transport.send(event2);

        // Restart server
        receivedData = [];
        server = net.createServer((socket) => {
            socket.on('data', (data) => {
                const lines = data.toString().split('\n').filter(line => line.trim() !== '');
                receivedData.push(...lines);
            });
        });

        await new Promise<void>((resolve) => {
            server.listen(TEST_SOCKET_PATH, resolve);
        });

        // Wait for reconnect and flush (backoff might delay this)
        // Transport has initial backoff 1000ms.
        await new Promise((resolve) => setTimeout(resolve, 2000));

        expect(receivedData.length).toBeGreaterThanOrEqual(2);
        const r1 = JSON.parse(receivedData[0]);
        const r2 = JSON.parse(receivedData[1]);

        // Order is preserved? Yes, buffer pushes.
        // It might be received in one chunk or separate.
        // If receivedData has 2 lines.

        // We need to parse all lines found
        const parsed = receivedData.map(d => JSON.parse(d));
        expect(parsed).toContainEqual(event1);
        expect(parsed).toContainEqual(event2);

        transport.dispose();
    });
});

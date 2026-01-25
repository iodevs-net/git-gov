import { test, expect } from 'vitest';
import * as net from 'net';
import * as fs from 'fs';
import { Transport } from '../src/transport';
import { SensorEvent } from '../src/types';

const SOCKET_PATH = '/tmp/cliff-watch-sensor.sock';

test('Integration: Transport sends data to socket', async () => {
    // Cleanup if exists
    if (fs.existsSync(SOCKET_PATH)) {
        try {
            fs.unlinkSync(SOCKET_PATH);
        } catch (e) {
            // ignore
        }
    }

    const receivedData: string[] = [];

    const server = net.createServer((socket) => {
        socket.on('data', (data) => {
            receivedData.push(data.toString());
        });
    });

    await new Promise<void>((resolve) => server.listen(SOCKET_PATH, resolve));

    const transport = new Transport();

    // Wait for connection
    await new Promise((resolve) => setTimeout(resolve, 500));

    const event: SensorEvent = {
        type: 'heartbeat',
        timestamp_ms: 1234567890
    };

    transport.send(event);

    // Wait for data
    await new Promise((resolve) => setTimeout(resolve, 500));

    transport.dispose();
    server.close();

    const combined = receivedData.join('');
    // Each event is new line delimited
    expect(combined).toContain(JSON.stringify(event) + '\n');

    // Cleanup
     if (fs.existsSync(SOCKET_PATH)) {
        try {
            fs.unlinkSync(SOCKET_PATH);
        } catch (e) {
            // ignore
        }
    }
});

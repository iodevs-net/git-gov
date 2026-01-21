import * as vscode from 'vscode';
import { SensorEvent, NavigationType } from './types';
import { Transport } from './transport';

let transport: Transport;

const sendEvent = (event: SensorEvent) => {
    transport.send(event);
};

let lastEditTimestamp = 0;
let editBurstBuffer = {
    chars_delta: 0,
    file_path: '',
    timestamp_ms: 0
};
let editBurstTimeout: NodeJS.Timeout | null = null;

const EDIT_BURST_DELAY_MS = 500;
const SCROLL_THROTTLE_MS = 500; // Arbitrary throttle for scroll

let lastScrollTimestamp = 0;

export function activate(context: vscode.ExtensionContext) {
    console.log('Git-Gov Witness is active');
    transport = new Transport();

    // Focus Tracking
    context.subscriptions.push(vscode.window.onDidChangeWindowState((windowState) => {
        const timestamp_ms = Date.now();
        if (windowState.focused) {
            const editor = vscode.window.activeTextEditor;
            const file_path = editor?.document.uri.fsPath || null;
            sendEvent({ type: 'focus_gained', file_path, timestamp_ms });
        } else {
            sendEvent({ type: 'focus_lost', timestamp_ms });
        }
    }));

    context.subscriptions.push(vscode.window.onDidChangeActiveTextEditor((editor) => {
        const timestamp_ms = Date.now();
        if (editor) {
            const file_path = editor.document.uri.fsPath;
            sendEvent({ type: 'focus_gained', file_path, timestamp_ms });
        }
    }));

    // Edit Tracking
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document.uri.scheme !== 'file') {return;}

        const timestamp_ms = Date.now();
        const delta = event.contentChanges.reduce((acc, change) => {
            return acc + change.text.length - change.rangeLength;
        }, 0);

        if (editBurstTimeout) {
            clearTimeout(editBurstTimeout);
        }

        // Check if we are continuing an existing burst for the same file
        if (editBurstBuffer.file_path === event.document.uri.fsPath && (timestamp_ms - lastEditTimestamp) < EDIT_BURST_DELAY_MS + 100) { // little buffer
             editBurstBuffer.chars_delta += delta;
             editBurstBuffer.timestamp_ms = timestamp_ms; // Update timestamp to latest? Or keep start? Roadmap implies "timestamp_ms" of the event.
             // Usually burst timestamp is when it happened. Let's say the timestamp is the end of the burst or the start?
             // "Timestamp Unix en milisegundos" - usually of the event creation.
             // I'll update it to now.
        } else {
            // Flush previous if exists and different file?
            // Actually, if we switch file, we should flush immediately?
            // But debouncing usually handles it. If I switch file, the previous timeout is still pending.
            // If I edit file A, then file B quickly.
            // The timeout is global here.

            // If file path changed, flush the previous one immediately
             if (editBurstBuffer.file_path && editBurstBuffer.file_path !== event.document.uri.fsPath && editBurstBuffer.chars_delta !== 0) {
                 sendEvent({
                     type: 'edit_burst',
                     file_path: editBurstBuffer.file_path,
                     chars_delta: editBurstBuffer.chars_delta,
                     timestamp_ms: editBurstBuffer.timestamp_ms
                 });
                 editBurstBuffer = { chars_delta: 0, file_path: '', timestamp_ms: 0 };
             }

             editBurstBuffer.file_path = event.document.uri.fsPath;
             editBurstBuffer.chars_delta += delta;
             editBurstBuffer.timestamp_ms = timestamp_ms;
        }

        lastEditTimestamp = timestamp_ms;

        editBurstTimeout = setTimeout(() => {
            if (editBurstBuffer.chars_delta !== 0) {
                sendEvent({
                    type: 'edit_burst',
                    file_path: editBurstBuffer.file_path,
                    chars_delta: editBurstBuffer.chars_delta,
                    timestamp_ms: editBurstBuffer.timestamp_ms
                });
                editBurstBuffer = { chars_delta: 0, file_path: '', timestamp_ms: 0 };
            }
        }, EDIT_BURST_DELAY_MS);
    }));

    // Navigation Tracking (Scroll)
    context.subscriptions.push(vscode.window.onDidChangeTextEditorVisibleRanges((event) => {
        const timestamp_ms = Date.now();
        if (timestamp_ms - lastScrollTimestamp > SCROLL_THROTTLE_MS) {
            sendEvent({
                type: 'navigation',
                file_path: event.textEditor.document.uri.fsPath,
                nav_type: 'scroll',
                timestamp_ms
            });
            lastScrollTimestamp = timestamp_ms;
        }
    }));

    // Heartbeat loop?
    // Roadmap 1.2 mentions heartbeat, but Phase 2 doesn't explicitly ask for it yet.
    // But Phase 2.1, 2.2, 2.3 are covered.
}

export function deactivate() {
    const timestamp_ms = Date.now();
    sendEvent({ type: 'disconnect', timestamp_ms });
    transport.dispose();
}

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
const transport_1 = require("./transport");
let transport;
function activate(context) {
    console.log('Cliff-Watch Witness is active');
    transport = new transport_1.Transport();
    context.subscriptions.push({ dispose: () => transport.dispose() });
    // 2.1 Focus Tracking
    context.subscriptions.push(vscode.window.onDidChangeWindowState((e) => {
        const timestamp_ms = Date.now();
        if (e.focused) {
            const editor = vscode.window.activeTextEditor;
            const file_path = editor ? editor.document.fileName : null;
            const event = { type: 'focus_gained', file_path, timestamp_ms };
            sendEvent(event);
        }
        else {
            const event = { type: 'focus_lost', timestamp_ms };
            sendEvent(event);
        }
    }), vscode.window.onDidChangeActiveTextEditor((editor) => {
        const timestamp_ms = Date.now();
        if (editor) {
            const file_path = editor.document.fileName;
            const event = { type: 'focus_gained', file_path, timestamp_ms };
            sendEvent(event);
        }
    }));
    // 2.2 Edit Tracking (Bursting)
    let editBurstAccumulator = 0;
    let editBurstTimeout = null;
    let currentEditFile = null;
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.uri.scheme !== 'file')
            return;
        // CNS v3.0 Pareto Filtering
        // Ignoramos Undo/Redo para no contaminar la métrica de originalidad humana
        if (e.reason === vscode.TextDocumentChangeReason.Undo || e.reason === vscode.TextDocumentChangeReason.Redo) {
            return;
        }
        const timestamp_ms = Date.now();
        const delta = e.contentChanges.reduce((acc, change) => {
            return acc + (change.text.length - change.rangeLength);
        }, 0);
        // CNS v3.0: Detección de Pegado Probable
        // Si un solo cambio inserta más de 30 caracteres, o si la ráfaga es sospechosamente rápida.
        const is_likely_paste = e.contentChanges.some(c => c.text.length > 30);
        // Enviamos evento de tecleo atómico solo si es un cambio pequeño (tecleo real)
        // para habilitar análisis de latencia física en el Daemon.
        if (e.contentChanges.length === 1 && e.contentChanges[0].text.length === 1 && !is_likely_paste) {
            sendEvent({
                type: 'keystroke',
                file_path: e.document.fileName,
                timestamp_ms,
                metadata: { char: e.contentChanges[0].text }
            });
        }
        if (editBurstTimeout) {
            clearTimeout(editBurstTimeout);
        }
        editBurstTimeout = setTimeout(() => {
            flushEditBurst();
        }, 500);
    }));
    function flushEditBurst() {
        if (editBurstAccumulator === 0 || !currentEditFile)
            return;
        const event = {
            type: 'edit_burst',
            file_path: currentEditFile,
            chars_delta: editBurstAccumulator,
            timestamp_ms: Date.now(),
            metadata: {
                is_likely_paste: editBurstAccumulator > 100 // Si la ráfaga acumulada es muy grande
            }
        };
        sendEvent(event);
        editBurstAccumulator = 0;
    }
    // 2.3 Navigation Tracking
    let lastScrollTime = 0;
    context.subscriptions.push(vscode.window.onDidChangeTextEditorVisibleRanges((e) => {
        const now = Date.now();
        if (now - lastScrollTime < 1000)
            return; // Throttle 1s
        sendEvent({
            type: 'navigation',
            file_path: e.textEditor.document.fileName,
            nav_type: 'scroll',
            timestamp_ms: now
        });
        lastScrollTime = now;
    }), 
    // Detección de Foco de Lectura por Hover
    vscode.languages.registerHoverProvider('*', {
        provideHover(document, position, token) {
            sendEvent({
                type: 'navigation',
                file_path: document.fileName,
                nav_type: 'hover',
                timestamp_ms: Date.now()
            });
            return null; // No interferimos con otros hovers
        }
    }));
    // Detección de Navegación (Go to Definition / Navegación interna)
    context.subscriptions.push(vscode.window.onDidChangeTextEditorSelection((e) => {
        if (e.kind === vscode.TextEditorSelectionChangeKind.Command) {
            // Probablemente un comando de navegación (Go to definition, etc)
            sendEvent({
                type: 'navigation',
                file_path: e.textEditor.document.fileName,
                nav_type: 'go_to_definition',
                timestamp_ms: Date.now()
            });
        }
    }));
    // Heartbeat v3.0: Identificamos si el sensor está "vivo" y en qué versión.
    const heartbeatInterval = setInterval(() => {
        sendEvent({
            type: 'heartbeat',
            timestamp_ms: Date.now()
        });
    }, 15000); // 15s para mayor resolución en el Daemon
    context.subscriptions.push({ dispose: () => clearInterval(heartbeatInterval) });
}
function deactivate() {
    sendEvent({ type: 'disconnect', timestamp_ms: Date.now() });
    if (transport) {
        transport.dispose();
    }
}
function sendEvent(event) {
    if (transport) {
        transport.send(event);
    }
}
//# sourceMappingURL=extension.js.map
import * as vscode from 'vscode';
import { SensorEvent, NavigationType } from './types';

import { Transport } from './transport';

let transport: Transport;

export function activate(context: vscode.ExtensionContext) {
  console.log('Cliff-Craft Witness is active');

  transport = new Transport();
  context.subscriptions.push({ dispose: () => transport.dispose() });

  // 2.1 Focus Tracking
  context.subscriptions.push(
    vscode.window.onDidChangeWindowState((e) => {
      const timestamp_ms = Date.now();
      if (e.focused) {
        const editor = vscode.window.activeTextEditor;
        const file_path = editor ? editor.document.fileName : null;
        const event: SensorEvent = { type: 'focus_gained', file_path, timestamp_ms };
        sendEvent(event);
      } else {
        const event: SensorEvent = { type: 'focus_lost', timestamp_ms };
        sendEvent(event);
      }
    }),

    vscode.window.onDidChangeActiveTextEditor((editor) => {
      const timestamp_ms = Date.now();
      if (editor) {
        const file_path = editor.document.fileName;
        const event: SensorEvent = { type: 'focus_gained', file_path, timestamp_ms };
        sendEvent(event);
      }
    })
  );

  // 2.2 Edit Tracking (Bursting)
  let editBurstAccumulator = 0;
  let editBurstTimeout: ReturnType<typeof setTimeout> | null = null;
  let currentEditFile: string | null = null;

  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document.uri.scheme !== 'file') return;

      // CNS v3.0 Pareto Filtering
      // Ignoramos Undo/Redo para no contaminar la métrica de originalidad humana
      if (e.reason === vscode.TextDocumentChangeReason.Undo || e.reason === vscode.TextDocumentChangeReason.Redo) {
        return;
      }

      const timestamp_ms = Date.now();
      const delta = e.contentChanges.reduce((acc, change) => {
        return acc + (change.text.length - change.rangeLength);
      }, 0);

      // Si el cambio es masivo (ej. pegado de código), marcamos como potencial no-humano
      const is_likely_paste = e.contentChanges.some(c => c.text.length > 50);

      if (delta === 0) return;

      if (currentEditFile !== e.document.fileName) {
        flushEditBurst();
        currentEditFile = e.document.fileName;
      }

      editBurstAccumulator += delta;

      // Enviar evento de tecleo atómico para análisis cinemático
      // Solo para cambios pequeños (tecleo real)
      if (delta === 1 && !is_likely_paste) {
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
    })
  );

  function flushEditBurst() {
    if (editBurstAccumulator === 0 || !currentEditFile) return;

    const event: SensorEvent = {
        type: 'edit_burst',
        file_path: currentEditFile,
        chars_delta: editBurstAccumulator,
        timestamp_ms: Date.now()
    };
    sendEvent(event);

    editBurstAccumulator = 0;
    // Keep file context until switched
  }


  // 2.3 Navigation Tracking
  let lastScrollTime = 0;
  context.subscriptions.push(
      vscode.window.onDidChangeTextEditorVisibleRanges((e) => {
          const now = Date.now();
          if (now - lastScrollTime < 1000) return; // Throttle 1s

          const event: SensorEvent = {
              type: 'navigation',
              file_path: e.textEditor.document.fileName,
              nav_type: 'scroll',
              timestamp_ms: now
          };
          sendEvent(event);
          lastScrollTime = now;
      })
  );

  // Heartbeat
  const heartbeatInterval = setInterval(() => {
      sendEvent({ type: 'heartbeat', timestamp_ms: Date.now() });
  }, 30000);
  context.subscriptions.push({ dispose: () => clearInterval(heartbeatInterval) });

}

export function deactivate() {
    sendEvent({ type: 'disconnect', timestamp_ms: Date.now() });
    if (transport) {
        transport.dispose();
    }
}

function sendEvent(event: SensorEvent) {
    if (transport) {
        transport.send(event);
    }
}

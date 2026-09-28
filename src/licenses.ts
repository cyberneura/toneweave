import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import './app.css';

const text = document.getElementById('licenses-text')!;

async function load() {
  try {
    text.textContent = await invoke<string>('third_party_notices');
  } catch (e) {
    text.textContent = `Could not load the license text: ${e}`;
  }
  // 本文を埋めてからウインドウを出してもらう (非表示で作ってある)
  await invoke('licenses_window_ready');
}

if (isTauri()) {
  load().catch((e: unknown) => console.error('toneweave: failed to show the licenses window', e));
  window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      event.preventDefault();
      void getCurrentWebviewWindow().close();
    }
  });
} else {
  text.textContent = 'Open the Toneweave desktop app to see the licenses. The same text is in THIRD-PARTY-NOTICES.txt.';
}

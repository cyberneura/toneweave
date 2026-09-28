import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
export default defineConfig({ plugins: [svelte(), tailwindcss()], clearScreen: false, server: { port: 1420, strictPort: true }, envPrefix: ['VITE_','TAURI_ENV_'], build: { target: 'safari14', modulePreload: { polyfill: false }, rollupOptions: { input: { main: 'index.html', licenses: 'licenses.html' } } } });

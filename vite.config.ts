import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

declare const process: {
    env: {
        TAURI_DEV_HOST?: string;
        TAURI_PLATFORM?: string;
        TAURI_DEBUG?: string;
    };
};

// Tauri expects a fixed port
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
    plugins: [sveltekit()],

    // Vite options tailored for Tauri development
    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                protocol: 'ws',
                host,
                port: 1421,
            }
            : undefined,
        watch: {
            // Tell Vite to ignore watching `src-tauri`
            ignored: ['**/src-tauri/**'],
        },
    },

    // Environment variables starting with TAURI_ are exposed to the browser
    envPrefix: ['VITE_', 'TAURI_'],

    build: {
        // Tauri uses Chromium on Windows and WebKit on macOS and Linux
        target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari14',
        // Don't minify for debug builds
        minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
        // Produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
    },
});

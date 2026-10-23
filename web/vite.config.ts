import { resolve } from 'node:path';
import react, { reactCompilerPreset } from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import babel from '@rolldown/plugin-babel';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

export default defineConfig({
	base: '/fend/',
	build: {
		minify: false,
		rollupOptions: {
			input: {
				main: resolve(import.meta.dirname, 'index.html'),
				widget: resolve(import.meta.dirname, 'widget.html'),
			},
		},
		sourcemap: true,
		target: 'esnext',
	},
	worker: {
		// eslint-disable-next-line @typescript-eslint/no-unsafe-return
		plugins: () => [wasm()],
		format: 'es',
	},
	plugins: [
		wasm(),
		react(),
		babel({
			presets: [reactCompilerPreset()],
		}),
	],
	server: {
		fs: {
			allow: [searchForWorkspaceRoot(process.cwd()), '../wasm/fend-wasm'],
		},
	},
});

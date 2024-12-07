import { resolve } from 'node:path';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

const ReactCompilerConfig = {};

export default defineConfig({
	base: '/fend/',
	build: {
		minify: false,
		rollupOptions: {
			input: {
				main: resolve(__dirname, 'index.html'),
				widget: resolve(__dirname, 'widget.html'),
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
		react({
			babel: {
				plugins: [['babel-plugin-react-compiler', ReactCompilerConfig]],
			},
		}),
	],
	server: {
		fs: {
			allow: [searchForWorkspaceRoot(process.cwd()), '../wasm/pkg-fend-web'],
		},
	},
});

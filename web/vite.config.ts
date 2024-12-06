import { resolve } from 'node:path';
import react from '@vitejs/plugin-react';
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
	plugins: [
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

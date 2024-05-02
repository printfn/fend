// vite.config.js
import { resolve } from 'node:path';
import react from '@vitejs/plugin-react-swc';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

export default defineConfig({
	base: '/fend/',
	build: {
		target: 'esnext',
		rollupOptions: {
			input: {
				main: resolve(__dirname, 'index.html'),
				widget: resolve(__dirname, 'widget.html'),
			},
		},
		sourcemap: true,
	},
	plugins: [react()],
	server: {
		fs: {
			allow: [searchForWorkspaceRoot(process.cwd()), '../wasm/pkg-fend-web'],
		},
	},
});

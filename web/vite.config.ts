// vite.config.js
import { resolve } from 'path';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

export default defineConfig({
	build: {
		rollupOptions: {
			input: {
				main: resolve(__dirname, 'index.html'),
				widget: resolve(__dirname, 'widget.html'),
			},
		},
	},
	base: '/fend/',
	server: {
		fs: {
			allow: [searchForWorkspaceRoot(process.cwd()), '../wasm/pkg-fend-web'],
		},
	},
});

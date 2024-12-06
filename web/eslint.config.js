// @ts-check

import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';
import reactCompiler from 'eslint-plugin-react-compiler';

/** @type any */
const reactHooksPlugin = reactHooks;
/** @type any */
const reactHooksRules = reactHooks.configs.recommended.rules;

export default tseslint.config(
	js.configs.recommended,
	...tseslint.configs.strictTypeChecked,
	reactRefresh.configs.vite,
	{ ignores: ['dist', 'cloudflare', 'eslint.config.js'] },
	{
		files: ['**/*.{ts,tsx}'],
	},
	{
		languageOptions: {
			globals: globals.browser,
			parserOptions: {
				projectService: true,
				tsconfigRootDir: import.meta.dirname,
			},
		},
		plugins: {
			'react-hooks': reactHooksPlugin,
			'react-compiler': reactCompiler,
		},
		rules: {
			...reactHooksRules,
			'react-compiler/react-compiler': 'error',
			'@typescript-eslint/promise-function-async': 'error',
		},
	},
);

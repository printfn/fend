// @ts-check

import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';

/** @type any */
const reactHooksPlugin = reactHooks;
/** @type any */
const reactHooksRules = reactHooks.configs.recommended.rules;

export default tseslint.config(
	{ ignores: ['dist'] },
	{
		extends: [js.configs.recommended, ...tseslint.configs.strictTypeChecked],
		files: ['**/*.{ts,tsx}'],
		languageOptions: {
			ecmaVersion: 2020,
			globals: globals.browser,
			parserOptions: {
				projectService: true,
				tsconfigRootDir: import.meta.dirname,
			},
		},
		plugins: {
			'react-hooks': reactHooksPlugin,
			'react-refresh': reactRefresh,
		},
		rules: {
			...reactHooksRules,
			'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
		},
	},
);

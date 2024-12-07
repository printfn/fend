import { context } from "esbuild";
import { wasmLoader } from 'esbuild-plugin-wasm';

const watch = process.argv.includes('--watch');

async function main() {
	const ctx = await context({
        entryPoints: ['index.ts'],
        bundle: true,
        outdir: 'dist',
        platform: 'node',
        format: 'esm',
        plugins: [wasmLoader()],
    });

	if (watch) {
		await ctx.watch();
	} else {
		await ctx.rebuild();
		await ctx.dispose();
	}
}

try {
	await main();
} catch (e) {
	console.error(e);
	process.exit(1);
};

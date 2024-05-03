import { evaluateFendWithVariablesJson, default as initWasm, initialiseWithHandlers } from 'fend-wasm';

export type FendArgs = {
	input: string;
	timeout: number;
	variables: string;
	currencyData: Map<string, number>;
};

export type FendResult = { ok: true; result: string; variables: string } | { ok: false; message: string };

self.addEventListener('message', async ({ data }: MessageEvent<FendArgs>) => {
	await initWasm();
	initialiseWithHandlers(data.currencyData);
	const result: FendResult = JSON.parse(evaluateFendWithVariablesJson(data.input, data.timeout, data.variables));
	postMessage(result);
});

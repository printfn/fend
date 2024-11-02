import { evaluateFendWithVariablesJson, default as initWasm, initialiseWithHandlers } from 'fend-wasm';

export type FendArgs = {
	input: string;
	timeout: number;
	variables: string;
	currencyData: Map<string, number>;
};

export type FendResult = { ok: true; result: string; variables: string } | { ok: false; message: string };

const eventListener = async ({ data }: MessageEvent<FendArgs>) => {
	await initWasm();
	initialiseWithHandlers(data.currencyData);
	const result = JSON.parse(evaluateFendWithVariablesJson(data.input, data.timeout, data.variables)) as FendResult;
	postMessage(result);
};
self.addEventListener('message', (ev: MessageEvent<FendArgs>) => void eventListener(ev));

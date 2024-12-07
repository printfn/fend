import { evaluateFendWithVariablesJson, initialiseWithHandlers } from 'fend-wasm';

export type FendArgs = {
	input: string;
	timeout: number;
	variables: string;
	currencyData: Map<string, number>;
};

export type FendResult = { ok: true; result: string; variables: string } | { ok: false; message: string };

function eventListener({ data }: MessageEvent<FendArgs>) {
	try {
		initialiseWithHandlers(data.currencyData);
		const result = JSON.parse(evaluateFendWithVariablesJson(data.input, data.timeout, data.variables)) as FendResult;
		postMessage(result);
	} catch (e: unknown) {
		console.error(e);
		throw e;
	}
}
self.addEventListener('message', (ev: MessageEvent<FendArgs>) => {
	eventListener(ev);
});
postMessage('ready');

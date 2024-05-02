import {
	evaluateFendWithTimeout,
	evaluateFendWithVariablesJson,
	default as initWasm,
	initialiseWithHandlers,
} from 'fend-wasm';
import { getExchangeRates } from './exchange-rates';

async function load() {
	try {
		const [, exchangeRates] = await Promise.all([initWasm(), getExchangeRates()]);
		initialiseWithHandlers(exchangeRates);

		const result = evaluateFendWithTimeout('1 + 2', 500);
		if (result !== '3') {
			alert('Failed to initialise WebAssembly');
			return;
		}
	} catch (e) {
		console.error(e);
		alert('Failed to initialise WebAssembly');
		return;
	}
}
await load();

type FendResult = { ok: true; result: string; variables: string } | { ok: false; message: string };

export async function fend(input: string, timeout: number, variables: string) {
	const res: FendResult = JSON.parse(evaluateFendWithVariablesJson(input, timeout, variables));
	return res;
}

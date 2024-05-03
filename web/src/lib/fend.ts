import { getExchangeRates } from './exchange-rates';
import type { FendArgs, FendResult } from './worker';
import MyWorker from './worker?worker';

let exchangeRateCache: Map<string, number> | null = null;
type WorkerCache = { worker: Worker; active: boolean; resolve: (fr: FendResult) => void };
let workerCache: WorkerCache | null = null;

export async function fend(input: string, timeout: number, variables: string): Promise<FendResult> {
	try {
		const currencyData = exchangeRateCache || (await getExchangeRates());
		exchangeRateCache = currencyData;
		return await new Promise<FendResult>((resolve, reject) => {
			const args: FendArgs = { input, timeout, variables, currencyData };
			const w = workerCache || { worker: new MyWorker(), active: false, resolve };
			if (w.active) {
				w.worker.terminate();
				w.resolve({ ok: false, message: 'cancelled' });
				w.worker = new MyWorker();
				w.active = false;
			}
			w.resolve = resolve;
			w.worker.onmessage = (e: MessageEvent<FendResult>) => {
				if (workerCache) {
					workerCache.active = false;
				}
				resolve(e.data);
			};
			w.worker.onerror = e => {
				if (workerCache) {
					workerCache.active = false;
				}
				reject(e);
			};
			workerCache = w;
			w.active = true;
			w.worker.postMessage(args);
		});
	} catch (e) {
		console.error(e);
		alert('Failed to initialise WebAssembly');
		return { ok: false, message: 'Failed to initialise WebAssembly' };
	}
}

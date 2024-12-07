import { getExchangeRates } from './exchange-rates';
import type { FendArgs, FendResult } from './worker';
import MyWorker from './worker?worker';

let exchangeRateCache: Map<string, number> | null = null;
type State = 'new' | 'ready' | 'busy';
class WorkerCache {
	worker!: Worker;
	state!: State;
	initialisedPromise!: Promise<void>;
	resolveDone?: (r: FendResult) => void;
	rejectError?: (e: Error) => void;

	init() {
		this.state = 'new';
		this.worker = new MyWorker({
			name: 'fend worker',
		});
		let resolveInitialised: () => void;
		this.initialisedPromise = new Promise<void>(resolve => {
			resolveInitialised = resolve;
		});
		this.worker.onmessage = (e: MessageEvent<FendResult | 'ready'>) => {
			this.state = 'ready';
			if (e.data === 'ready') {
				resolveInitialised();
			} else {
				this.resolveDone?.(e.data);
			}
		};
		this.worker.onerror = e => {
			this.state = 'ready';
			this.rejectError?.(new Error(e.message, { cause: e }));
		};
		this.worker.onmessageerror = e => {
			this.state = 'ready';
			this.rejectError?.(new Error('received messageerror event', { cause: e }));
		};
	}

	constructor() {
		this.init();
	}

	async query(args: FendArgs) {
		if (this.state === 'new') {
			await this.initialisedPromise;
		}
		if (this.state === 'busy') {
			console.log('terminating existing worker');
			this.worker.terminate();
			this.resolveDone?.({ ok: false, message: 'cancelled' });
			this.init();
			await this.initialisedPromise;
		}
		if (this.state !== 'ready') {
			throw new Error('unexpected worker state: ' + this.state);
		}
		const p = new Promise<FendResult>((resolve, reject) => {
			this.resolveDone = resolve;
			this.rejectError = reject;
		});
		this.state = 'busy';
		this.worker.postMessage(args);
		return await p;
	}
}
const workerCache = new WorkerCache();

export async function fend(input: string, timeout: number, variables: string): Promise<FendResult> {
	try {
		const currencyData = exchangeRateCache || (await getExchangeRates());
		exchangeRateCache = currencyData;
		const args: FendArgs = { input, timeout, variables, currencyData };
		return await workerCache.query(args);
	} catch (e) {
		console.error(e);
		alert('Failed to initialise WebAssembly');
		return { ok: false, message: 'Failed to initialise WebAssembly' };
	}
}

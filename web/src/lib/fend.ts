import { getExchangeRates } from './exchange-rates';
import type { FendArgs, FendResult } from './worker';
import MyWorker from './worker?worker';

function newAbortError(message: string) {
	const e = new Error(message);
	e.name = 'AbortError';
	return e;
}

type State = 'new' | 'ready' | 'busy';
type WorkerCache = {
	worker: Worker;
	state: State;
	initialisedPromise: Promise<void>;
	resolveDone?: (r: FendResult) => void;
	rejectError?: (e: Error) => void;
};

function init() {
	let resolveInitialised: () => void;
	const result: WorkerCache = {
		state: 'new',
		worker: new MyWorker({
			name: 'fend worker',
		}),
		initialisedPromise: new Promise<void>(resolve => {
			resolveInitialised = resolve;
		}),
	};
	result.worker.onmessage = (e: MessageEvent<FendResult | 'ready'>) => {
		result.state = 'ready';
		if (e.data === 'ready') {
			resolveInitialised();
		} else {
			result.resolveDone?.(e.data);
		}
	};
	result.worker.onerror = e => {
		result.state = 'ready';
		result.rejectError?.(new Error(e.message, { cause: e }));
	};
	result.worker.onmessageerror = e => {
		result.state = 'ready';
		result.rejectError?.(new Error('received messageerror event', { cause: e }));
	};
	return result;
}

let workerCache: WorkerCache = init();
let id = 0;

async function query(args: FendArgs) {
	let w = workerCache;
	const i = ++id;
	if (w.state === 'new') {
		await w.initialisedPromise;
		if (i < id) {
			throw newAbortError('created new worker during initialisation');
		}
	}
	if (w.state === 'busy') {
		console.log('terminating existing worker');
		w.worker.terminate();
		w.resolveDone?.({ ok: false, message: 'cancelled' });
		w = init();
		workerCache = w;
		await w.initialisedPromise;
		if (i < id) {
			throw newAbortError('created new worker while worker was busy');
		}
	}
	if (w.state !== 'ready') {
		throw new Error('unexpected worker state: ' + w.state);
	}
	const p = new Promise<FendResult>((resolve, reject) => {
		w.resolveDone = resolve;
		w.rejectError = reject;
	});
	w.state = 'busy';
	w.worker.postMessage(args);
	return await p;
}

export async function fend(input: string, timeout: number, variables: string): Promise<FendResult> {
	try {
		const currencyData = await getExchangeRates();
		const args: FendArgs = { input, timeout, variables, currencyData };
		return await query(args);
	} catch (e) {
		if (e instanceof Error && e.name === 'AbortError') {
			return { ok: false, message: 'Aborted' };
		}
		console.error(e);
		alert('Failed to initialise WebAssembly');
		return { ok: false, message: 'Failed to initialise WebAssembly' };
	}
}

export class WaitGroup {
	#counter = 0;
	#promise = Promise.resolve();
	#resolve?: () => void;

	enter() {
		if (this.#counter === 0) {
			this.#promise = new Promise(resolve => {
				this.#resolve = resolve;
			});
		}
		++this.#counter;
	}

	leave() {
		if (this.#counter <= 0 || !this.#resolve) {
			throw new Error('leave() called without a matching enter()');
		}
		--this.#counter;
		if (this.#counter === 0) {
			this.#resolve();
			this.#resolve = undefined;
		}
	}

	async wait(abortSignal?: AbortSignal) {
		if (abortSignal) {
			await abortPromise(abortSignal, this.#promise);
		} else {
			await this.#promise;
		}
	}

	get counter() {
		return this.#counter;
	}
}

async function abortPromise<T>(abortSignal: AbortSignal, promise: Promise<T>) {
	return new Promise<T>((resolve, reject) => {
		if (abortSignal.aborted) {
			reject(abortSignal.reason as Error);
			return;
		}

		const onAbort = () => {
			cleanup();
			reject(abortSignal.reason as Error);
		};

		const cleanup = () => {
			abortSignal.removeEventListener('abort', onAbort);
		};

		abortSignal.addEventListener('abort', onAbort);

		promise.then(
			value => {
				cleanup();
				resolve(value);
			},
			(error: unknown) => {
				cleanup();
				reject(error as Error);
			},
		);
	});
}

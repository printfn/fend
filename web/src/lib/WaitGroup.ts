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
			await abortPromise(abortSignal, async () => {
				await this.#promise;
			});
		} else {
			await this.#promise;
		}
	}

	get counter() {
		return this.#counter;
	}
}

async function abortPromise(abortSignal: AbortSignal, f: () => Promise<void>) {
	return new Promise((resolve, reject) => {
		if (abortSignal.aborted) {
			// eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
			reject(abortSignal.reason);
			return;
		}

		const onAbort = () => {
			cleanup();
			// eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
			reject(abortSignal.reason);
		};

		const cleanup = () => {
			abortSignal.removeEventListener('abort', onAbort);
		};

		abortSignal.addEventListener('abort', onAbort);

		f().then(
			value => {
				cleanup();
				resolve(value);
			},
			(error: unknown) => {
				cleanup();
				// eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
				reject(error);
			},
		);
	});
}

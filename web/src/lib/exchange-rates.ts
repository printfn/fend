import { WaitGroup } from './WaitGroup';

const wg = new WaitGroup();
let exchangeRateCache: Map<string, number> | undefined;

async function fetchExchangeRates() {
	try {
		const map = new Map<string, number>();
		const res = await fetch('https://fend.pr.workers.dev/exchange-rates');
		const xml = await res.text();
		const dom = new DOMParser().parseFromString(xml, 'text/xml');

		for (const node of dom.querySelectorAll('UN_OPERATIONAL_RATES')) {
			const currency = node.querySelector('f_curr_code')?.textContent;
			if (!currency) continue;
			const rateStr = node.querySelector('rate')?.textContent;
			if (!rateStr) continue;
			const rate = Number.parseFloat(rateStr);

			if (!Number.isNaN(rate) && Number.isFinite(rate)) {
				map.set(currency, rate);
			}
		}
		return map;
	} catch (e) {
		throw new Error('failed to fetch currencies', { cause: e });
	}
}

export async function getExchangeRates(): Promise<Map<string, number>> {
	await wg.wait();

	try {
		wg.enter();
		if (exchangeRateCache) {
			return exchangeRateCache;
		}
		exchangeRateCache = await fetchExchangeRates();
		return exchangeRateCache;
	} catch (e) {
		console.log(e);
		exchangeRateCache = new Map();
		return exchangeRateCache;
	} finally {
		wg.leave();
	}
}

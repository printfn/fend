export async function getExchangeRates() {
	const map = new Map<string, number>();

	try {
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
	} catch (e) {
		console.error('Failed to fetch currencies', e);
	}

	return map;
}

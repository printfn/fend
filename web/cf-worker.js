// This needs to be manually deployed to Cloudflare

export default {
	async fetch(request, env, ctx) {
		if (
			request.url === 'https://fend.pr.workers.dev/exchange-rates' &&
			request.method === 'GET'
		) {
			const fetchResult = await fetch(
				'https://treasury.un.org/operationalrates/xsql2XML.php',
				{
					cf: {
						cacheTtl: 86400,
						cacheEverything: true,
					},
					headers: {
						'X-Source': 'Cloudflare-Workers',
					},
				},
			);
			const data = await fetchResult.text();
			return new Response(data, {
				headers: {
					'Access-Control-Allow-Origin': 'https://printfn.github.io',
					'Cache-Control': 'max-age=172800',
				},
			});
		} else {
			return new Response(null, {
				status: 404,
			});
		}
	},
};

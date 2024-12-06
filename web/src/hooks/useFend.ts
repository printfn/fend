import { useCallback, useState } from 'react';
import { fend } from '../lib/fend';

export function useFend() {
	const [variables, setVariables] = useState('');

	const evaluate = useCallback(
		async (input: string) => {
			const result = await fend(input, 1000000000, variables);
			console.log(result);
			if (result.ok && result.variables.length > 0) {
				setVariables(result.variables);
			}
			return result;
		},
		[variables],
	);

	const evaluateHint = useCallback(
		async (input: string) => {
			const result = await fend(input, 100, variables);
			if (!result.ok) {
				return '';
			} else {
				return result.result;
			}
		},
		[variables],
	);

	return { evaluate, evaluateHint };
}

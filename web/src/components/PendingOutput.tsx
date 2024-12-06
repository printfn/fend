import { startTransition, useEffect, useState, type Ref } from 'react';
import { ThreeDotsScale } from 'react-svg-spinners';
import { fend } from '../lib/fend';

type Props = {
	ref: Ref<HTMLParagraphElement>;
	currentInput: string;
	variables: string;
	isPending: boolean;
};

export default function PendingOutput({ ref, currentInput, variables, isPending }: Props) {
	const [hint, setHint] = useState('');
	useEffect(() => {
		startTransition(async () => {
			const result = await fend(currentInput, 100, variables);
			if (!result.ok) {
				setHint('');
			} else {
				setHint(result.result);
			}
		});
	}, [currentInput, variables]);

	return (
		<p id="pending-output" ref={ref}>
			{hint || (isPending ? <ThreeDotsScale /> : <>&nbsp;</>)}
		</p>
	);
}

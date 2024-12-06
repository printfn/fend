import type { Ref } from 'react';
import { ThreeDotsScale } from 'react-svg-spinners';

type Props = {
	ref: Ref<HTMLParagraphElement>;
	hint: string;
	isPending: boolean;
};

export default function PendingOutput({ ref, hint, isPending }: Props) {
	return (
		<p id="pending-output" ref={ref}>
			{hint || (isPending ? <ThreeDotsScale /> : <>&nbsp;</>)}
		</p>
	);
}

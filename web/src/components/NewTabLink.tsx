import type { ReactNode } from 'react';

type Props = {
	children: ReactNode;
	href: string;
};

export default function NewTabLink({ children, href }: Props) {
	return (
		<a rel="noreferrer noopener" target="_blank" href={`https://${href}`}>
			{children}
		</a>
	);
}

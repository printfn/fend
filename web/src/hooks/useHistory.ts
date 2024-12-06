import { useCallback, useState } from 'react';

const initialHistory = JSON.parse(localStorage.getItem('fend_history') || '[]') as string[];

export function useHistory() {
	const [history, setHistory] = useState<string[]>(initialHistory);

	const addToHistory = useCallback((newEntry: string) => {
		if (newEntry.startsWith(' ')) return;
		if (newEntry.trim().length === 0) return;
		setHistory(prevHistory => {
			const updatedHistory = [...prevHistory, newEntry]
				.filter((entry, idx, array) => idx === 0 || entry !== array[idx - 1]);
			localStorage.setItem('fend_history', JSON.stringify(updatedHistory.slice(-100)));
			return updatedHistory;
		});
	}, []);

	return { history, addToHistory };
}

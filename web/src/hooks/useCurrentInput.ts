import { FormEvent, useCallback, useState } from 'react';
import { useHistory } from './useHistory';

export function useCurrentInput() {
	const { history, addToHistory } = useHistory();
	const [currentInput, setCurrentInput] = useState('');
	const [navigation, setNavigation] = useState(0);

	const navigate = useCallback(
		(direction: 'up' | 'down') => {
			setNavigation(n => {
				let newValue: number;
				switch (direction) {
					case 'up':
						newValue = Math.min(n + 1, history.length);
						break;
					case 'down':
						newValue = Math.max(n - 1, 0);
						break;
				}
				if (newValue > 0) {
					setCurrentInput(history[history.length - newValue]);
				}
				if (newValue === 0) {
					setCurrentInput('');
				}
				return newValue;
			});
		},
		[history],
	);

	const onInput = useCallback((e: string | FormEvent<HTMLTextAreaElement>) => {
		setNavigation(0);
		setCurrentInput(typeof e === 'string' ? e : e.currentTarget.value);
	}, []);

	const upArrow = useCallback(() => {
		if (currentInput.trim().length !== 0 && navigation === 0) {
			// todo we should allow navigating history if input has been typed
			return;
		}
		navigate('up');
	}, [currentInput, navigate, navigation]);

	const downArrow = useCallback(() => {
		navigate('down');
	}, [navigate]);

	const submit = useCallback(() => {
		addToHistory(currentInput);
		setNavigation(0);
	}, [currentInput, addToHistory]);

	return { currentInput, submit, onInput, downArrow, upArrow };
}

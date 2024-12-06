import { type FormEvent, type KeyboardEvent, type ReactNode, useCallback, useEffect, useRef, useState } from 'react';
import { ThreeDotsScale } from 'react-svg-spinners';
import { fend } from './lib/fend';

const examples = `
> 5'10" to cm
177.8 cm

> cos (pi/4) + i * (sin (pi/4))
approx. 0.7071067811 + 0.7071067811i

> 0b1001 + 3
0b1100

> 0xffff to decimal
65535

> 100 °C to °F
212 °F

> 1 lightyear to parsecs
approx. 0.3066013937 parsecs

`;

const exampleContent = (
	<p id="examples">
		{'\n'}
		<b>examples:</b>
		{examples}
		<b>give it a go:</b>
	</p>
);

function NewTabLink({ children, href }: { children: ReactNode; href: string }) {
	return (
		<a rel="noreferrer noopener" target="_blank" href={`https://${href}`}>
			{children}
		</a>
	);
}

const initialHistory = JSON.parse(localStorage.getItem('fend_history') || '[]') as string[];

export default function App({ widget = false }: { widget?: boolean }) {
	const [currentInput, setCurrentInput] = useState('');
	const [output, setOutput] = useState<ReactNode>(widget ? <></> : exampleContent);
	const [history, setHistory] = useState<string[]>(initialHistory);
	useEffect(() => {
		const history100 = history.slice(-100);
		localStorage.setItem('fend_history', JSON.stringify(history100));
	}, [history]);
	const [variables, setVariables] = useState('');
	const [navigation, setNavigation] = useState(0);
	const [hint, setHint] = useState('');
	const [pending, setPending] = useState(0);
	useEffect(() => {
		void (async () => {
			const result = await fend(currentInput, 100, variables);
			if (!result.ok) {
				setHint('');
			} else {
				setHint(result.result);
			}
		})();
	}, [currentInput, variables]);
	const inputText = useRef<HTMLTextAreaElement>(null);
	const inputHint = useRef<HTMLParagraphElement>(null);
	const focus = useCallback(() => {
		// allow the user to select text for copying and
		// pasting, but if text is deselected (collapsed)
		// refocus the input field
		if (document.activeElement !== inputText.current && document.getSelection()?.isCollapsed) {
			inputText.current?.focus();
		}
	}, []);
	const update = useCallback((e: FormEvent<HTMLTextAreaElement>) => {
		setCurrentInput(e.currentTarget.value);
		setNavigation(0);
	}, []);
	const evaluate = useCallback(
		(event: KeyboardEvent<HTMLTextAreaElement>) => {
			void (async () => {
				// allow multiple lines to be entered if shift, ctrl
				// or meta is held, otherwise evaluate the expression
				if (!(event.key === 'Enter' && !event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey)) {
					return;
				}
				event.preventDefault();
				if (currentInput.trim() === 'clear') {
					setCurrentInput('');
					setOutput(null);
					return;
				}
				const request = <p>{`> ${currentInput}`}</p>;
				if (currentInput.trim().length > 0) {
					setHistory(h => [...h, currentInput]);
				}
				setNavigation(0);
				setPending(p => p + 1);
				const fendResult = await fend(currentInput, 1000000000, variables);
				setPending(p => p - 1);
				if (!fendResult.ok && fendResult.message === 'cancelled') {
					return;
				}
				setCurrentInput('');
				console.log(fendResult);
				const result = <p>{fendResult.ok ? fendResult.result : fendResult.message}</p>;
				if (fendResult.ok && fendResult.variables.length > 0) {
					setVariables(fendResult.variables);
				}
				setOutput(o => (
					<>
						{o}
						{request}
						{result}
					</>
				));
				inputHint.current?.scrollIntoView();
			})();
		},
		[currentInput, variables],
	);
	const navigate = useCallback(
		(event: KeyboardEvent<HTMLTextAreaElement>) => {
			if (
				(event.key === 'k' && event.metaKey !== event.ctrlKey && !event.altKey) ||
				(event.key === 'l' && event.ctrlKey && !event.metaKey && !event.altKey)
			) {
				// Cmd+K, Ctrl+K or Ctrl+L to clear the buffer
				setOutput(null);
				return;
			}
			if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') {
				return;
			}
			if (navigation > 0) {
				event.preventDefault();
				if (event.key === 'ArrowUp') {
					setNavigation(n => Math.min(n + 1, history.length));
				} else {
					setNavigation(n => Math.max(n - 1, 0));
					setCurrentInput('');
				}
			} else if (currentInput.trim().length === 0 && event.key === 'ArrowUp' && history.length > 0) {
				event.preventDefault();
				setNavigation(1);
			}
		},
		[currentInput, navigation, history],
	);
	useEffect(() => {
		document.addEventListener('click', focus);
		return () => {
			document.removeEventListener('click', focus);
		};
	}, [focus]);
	useEffect(() => {
		if (navigation > 0) {
			setCurrentInput(history[history.length - navigation]);
		}
	}, [navigation, history]);
	return (
		<main>
			{!widget && (
				<h1 id="about">
					<NewTabLink href="printfn.github.io/fend/documentation/">fend</NewTabLink> is an arbitrary-precision
					unit-aware calculator.
				</h1>
			)}
			<div id="output">{output}</div>
			<div id="input">
				<div id="text">
					<textarea
						autoComplete="off"
						autoCorrect="off"
						autoCapitalize="none"
						spellCheck="false"
						id="input-text"
						rows={currentInput.split('\n').length}
						ref={inputText}
						value={currentInput}
						onInput={update}
						onKeyPress={evaluate}
						onKeyDown={navigate}
						autoFocus
					/>
				</div>
				<p id="input-hint" ref={inputHint}>
					{hint || (pending > 0 ? <ThreeDotsScale /> : null)}
				</p>
			</div>
		</main>
	);
}

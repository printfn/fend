import {
	evaluateFendWithTimeout,
	evaluateFendWithVariablesJson,
	default as initWasm,
	initialiseWithHandlers,
} from 'fend-wasm';
import {
	type FormEvent,
	type KeyboardEvent,
	type ReactNode,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react';
import { getExchangeRates } from './lib/exchange-rates';

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

async function load() {
	try {
		await initWasm();
		initialiseWithHandlers(await getExchangeRates());

		const result = evaluateFendWithTimeout('1 + 2', 500);
		if (result !== '3') {
			alert('Failed to initialise WebAssembly');
			return;
		}
	} catch (e) {
		console.error(e);
		alert('Failed to initialise WebAssembly');
		return;
	}
}
await load();

export default function App({ widget = false }: { widget?: boolean }) {
	const [currentInput, setCurrentInput] = useState('');
	const [output, setOutput] = useState<ReactNode>(widget ? <></> : exampleContent);
	const [history, setHistory] = useState<string[]>([]);
	const [variables, setVariables] = useState('');
	const [navigation, setNavigation] = useState(0);
	const hint = useMemo<string>(() => {
		const result = JSON.parse(evaluateFendWithVariablesJson(currentInput, 100, variables));
		if (!result.ok) {
			return '';
		}
		return result.result;
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
			// allow multiple lines to be entered if shift, ctrl
			// or meta is held, otherwise evaluate the expression
			if (!(event.key === 'Enter' && !event.shiftKey && !event.ctrlKey && !event.metaKey)) {
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
			const fendResult = JSON.parse(evaluateFendWithVariablesJson(currentInput, 500, variables));
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
		},
		[currentInput, variables],
	);
	const navigate = useCallback(
		(event: KeyboardEvent<HTMLTextAreaElement>) => {
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
		(async () => {
			await load();
		})();
	}, []);
	useEffect(() => {
		document.addEventListener('click', focus);
		return () => {
			document.removeEventListener('click', focus);
		};
	});
	useEffect(() => {
		if (navigation > 0) {
			setCurrentInput(history[history.length - navigation]);
		}
	}, [navigation, history]);
	return (
		<main>
			{!widget && (
				<h1 id="about">
					<a rel="noreferrer noopener" target="_blank" href="https://printfn.github.io/fend/documentation/">
						fend
					</a>{' '}
					is an arbitrary-precision unit-aware calculator.
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
						rows={1}
						ref={inputText}
						value={currentInput}
						onInput={update}
						onKeyPress={evaluate}
						onKeyDown={navigate}
						// biome-ignore lint/a11y/noAutofocus:
						autoFocus
					/>
				</div>
				<p id="input-hint" ref={inputHint}>
					{hint}
				</p>
			</div>
		</main>
	);
}

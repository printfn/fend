import { type KeyboardEvent, type ReactNode, useCallback, useEffect, useRef, useState, useTransition } from 'react';
import { useCurrentInput } from './hooks/useCurrentInput';
import NewTabLink from './components/NewTabLink';
import PendingOutput from './components/PendingOutput';
import { useFend } from './hooks/useFend';

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

export default function App({ widget = false }: { widget?: boolean }) {
	const [output, setOutput] = useState<ReactNode>(widget ? <></> : exampleContent);
	const { evaluate, evaluateHint } = useFend();
	const { currentInput, submit, onInput, upArrow, downArrow, hint } = useCurrentInput(evaluateHint);
	const inputText = useRef<HTMLTextAreaElement>(null);
	const pendingOutput = useRef<HTMLParagraphElement>(null);

	const [isPending, startTransition] = useTransition();
	const onKeyDown = useCallback(
		(event: KeyboardEvent<HTMLTextAreaElement>) => {
			if (
				(event.key === 'k' && event.metaKey !== event.ctrlKey && !event.altKey) ||
				(event.key === 'l' && event.ctrlKey && !event.metaKey && !event.altKey)
			) {
				// Cmd+K, Ctrl+K or Ctrl+L to clear the buffer
				setOutput(null);
				return;
			}
			if (event.key === 'ArrowUp') {
				event.preventDefault();
				upArrow();
				return;
			}
			if (event.key === 'ArrowDown') {
				event.preventDefault();
				downArrow();
				return;
			}

			// allow multiple lines to be entered if shift, ctrl
			// or meta is held, otherwise evaluate the expression
			if (!(event.key === 'Enter' && !event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey)) {
				return;
			}
			event.preventDefault();
			if (currentInput.trim() === 'clear') {
				onInput('');
				setOutput(null);
				return;
			}

			startTransition(async () => {
				const request = <p>{`> ${currentInput}`}</p>;
				submit();
				const fendResult = await evaluate(currentInput);
				if (!fendResult.ok && fendResult.message === 'cancelled') {
					return;
				}
				onInput('');
				const result = <p>{fendResult.ok ? fendResult.result : `Error: ${fendResult.message}`}</p>;
				setOutput(o => (
					<>
						{o}
						{request}
						{result}
					</>
				));
				setTimeout(() => {
					pendingOutput.current?.scrollIntoView({ behavior: 'smooth' });
				}, 50);
			});
		},
		[currentInput, submit, onInput, downArrow, upArrow, evaluate],
	);
	useEffect(() => {
		const focus = () => {
			// allow the user to select text for copying and
			// pasting, but if text is deselected (collapsed)
			// refocus the input field
			if (document.activeElement !== inputText.current && document.getSelection()?.isCollapsed) {
				inputText.current?.focus();
			}
		};
		document.addEventListener('click', focus);
		return () => {
			document.removeEventListener('click', focus);
		};
	}, []);
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
						onInput={onInput}
						onKeyDown={onKeyDown}
						autoFocus
					/>
				</div>
				<PendingOutput ref={pendingOutput} hint={hint} isPending={isPending} />
			</div>
		</main>
	);
}

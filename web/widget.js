const {
	initialise,
	initialiseWithHandlers,
	evaluateFendWithTimeout,
	evaluateFendWithVariablesJson,
} = wasm_bindgen;

const EVALUATE_KEY = 13;
const NAVIGATE_UP_KEY = 38;
const NAVIGATE_DOWN_KEY = 40;

let output = document.getElementById('output');
let inputText = document.getElementById('input-text');
let inputHint = document.getElementById('input-hint');
let wasmInitialised = false;
let history = [];
let variables = '';
let navigation = 0;

async function evaluate(event) {
	// allow multiple lines to be entered if shift, ctrl
	// or meta is held, otherwise evaluate the expression
	if (
		!(
			event.keyCode == EVALUATE_KEY &&
			!event.shiftKey &&
			!event.ctrlKey &&
			!event.metaKey
		)
	) {
		return;
	}

	event.preventDefault();

	if (inputText.value == 'clear') {
		output.innerHTML = '';
		inputText.value = '';
		inputHint.innerText = '';
		return;
	}

	let request = document.createElement('p');
	let result = document.createElement('p');

	request.innerText = '> ' + inputText.value;

	if (isInputFilled()) {
		history.push(inputText.value);
	}

	navigateEnd();

	const fendResult = JSON.parse(
		evaluateFendWithVariablesJson(inputText.value, 500, variables),
	);

	inputText.value = '';
	inputHint.innerText = '';

	console.log(fendResult);

	result.innerText = fendResult.ok ? fendResult.result : fendResult.message;
	if (fendResult.ok && fendResult.variables.length > 0) {
		variables = fendResult.variables;
	}

	output.appendChild(request);
	output.appendChild(result);

	inputHint.scrollIntoView();
}

function navigate(event) {
	if (![NAVIGATE_UP_KEY, NAVIGATE_DOWN_KEY].includes(event.keyCode)) {
		return;
	}
	if (navigation > 0) {
		if (NAVIGATE_UP_KEY == event.keyCode) {
			event.preventDefault();

			navigateBackwards();
		} else if (NAVIGATE_DOWN_KEY == event.keyCode) {
			event.preventDefault();

			navigateForwards();
		}
	} else if (
		!isInputFilled() &&
		history.length > 0 &&
		NAVIGATE_UP_KEY == event.keyCode
	) {
		event.preventDefault();

		navigateBegin();
	}

	if (navigation > 0) {
		navigateSet();
	}

	updateReplicatedText();
	updateHint();
}

function navigateBackwards() {
	navigation += 1;

	if (navigation > history.length) {
		navigation = history.length;
	}
}

function navigateForwards() {
	navigation -= 1;

	if (navigation < 1) {
		navigateEnd();
		navigateClear();
	}
}

function navigateBegin() {
	navigation = 1;
}

function navigateEnd() {
	navigation = 0;
}

function navigateSet() {
	inputText.value = history[history.length - navigation];
}

function navigateClear() {
	inputText.value = '';
}

function focus() {
	// allow the user to select text for copying and
	// pasting, but if text is deselected (collapsed)
	// refocus the input field
	if (
		document.activeElement != inputText &&
		document.getSelection().isCollapsed
	) {
		inputText.focus();
	}
}

async function update() {
	updateReplicatedText();
	navigateEnd();
	updateHint();
}

function updateReplicatedText() {
	inputText.parentNode.dataset.replicatedValue = inputText.value;
}

function updateHint() {
	const result = JSON.parse(
		evaluateFendWithVariablesJson(inputText.value, 100, variables),
	);

	if (result.ok) {
		inputHint.innerText = result.result;
	} else {
		inputHint.innerText = '';
	}
}

function isInputFilled() {
	return inputText.value.length > 0;
}

async function getExchangeRates() {
	const map = new Map();

	try {
		const res = await fetch('https://fend.pr.workers.dev/exchange-rates');
		const xml = await res.text();
		const dom = new DOMParser().parseFromString(xml, 'text/xml');

		dom.querySelectorAll('UN_OPERATIONAL_RATES').forEach(node => {
			const currency = node.querySelector('f_curr_code').textContent;
			const rate = parseFloat(node.querySelector('rate').textContent);

			if (!Number.isNaN(rate) && Number.isFinite(rate)) {
				map.set(currency, rate);
			}
		});
	} catch (_) {}

	return map;
}

async function load() {
	try {
		await wasm_bindgen('./pkg/fend_wasm_bg.wasm');
		initialiseWithHandlers(await getExchangeRates());

		const result = evaluateFendWithTimeout('1 + 2', 500);
		if (result !== '3') {
			alert('Failed to initialise WebAssembly');
			return;
		}
	} catch (e) {
		alert('Failed to initialise WebAssembly');
		return;
	}
	wasmInitialised = true;

	inputText.addEventListener('input', update);
	inputText.addEventListener('keypress', evaluate);
	inputText.addEventListener('keydown', navigate);
	document.addEventListener('click', focus);
}

window.onload = load;

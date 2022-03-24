let output = document.getElementById("output");
let input_text = document.getElementById("input-text");
let input_hint = document.getElementById("input-hint");
let wasmInitialised = false;
let history = [];

async function evaluateFend(input) {
    const { initialise, evaluateFendWithTimeout, evaluateFendWithTimeoutMultiple } = wasm_bindgen;

    if (!wasmInitialised) {
        await wasm_bindgen('./pkg/fend_wasm_bg.wasm');

        initialise();

        evaluateFendWithTimeout("1 + 2", 500);
        wasmInitialised = true;
    }

    return evaluateFendWithTimeoutMultiple(input, 500);
}

async function submit(event) {
    if (event.keyCode == 13 && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
        const { evaluateFendWithTimeoutMultiple } = wasm_bindgen;

        event.preventDefault();

        let request = document.createElement("p");
        let result = document.createElement("p");

        request.innerText = "> " + input_text.value;

        history.push(input_text.value);

        input_text.value = "";
        input_hint.innerText = "";

        let results = evaluateFendWithTimeoutMultiple(history.join('\0'), 500).split('\0');

        result.innerText = results[results.length - 1];

        output.appendChild(request);
        output.appendChild(result);

        input_hint.scrollIntoView();
    }
}

function focus() {
    // allow the user to select text for copying and
    // pasting, but if text is deselected (collapsed)
    // refocus the input field
    if (document.activeElement != input_text && document.getSelection().isCollapsed) {
        input_text.focus();
    }
}

async function update() {
    input_text.parentNode.dataset.replicatedValue = input_text.value;

    let result = await evaluateFend(input_text.value);

    if (!result.startsWith('Error: ')) {
        input_hint.innerText = result;
    } else {
        input_hint.innerText = "";
    }
}

async function load() {
    const { initialise, evaluateFendWithTimeout } = wasm_bindgen;

    await wasm_bindgen('./pkg/fend_wasm_bg.wasm');

    initialise();

    evaluateFendWithTimeout("1 + 2", 500);
    wasmInitialised = true;

    input_text.addEventListener('input', update);
    input_text.addEventListener('keypress', submit);
    document.addEventListener('click', focus)
};

window.onload = load;

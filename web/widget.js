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
    const { evaluateFendWithTimeoutMultiple } = wasm_bindgen;
    let input = document.getElementById("input-text");

    if (event.keyCode == 13 && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();

        let hint = document.getElementById("input-hint");
        let output = document.getElementById("output");
        let request = document.createElement("p");
        let result = document.createElement("p");

        request.innerText = "> " + input.value;

        history.push(input.value);

        input.value = "";
        hint.innerText = "";

        let results = evaluateFendWithTimeoutMultiple(history.join('\0'), 500).split('\0');

        result.innerText = results[results.length - 1];

        output.appendChild(request);
        output.appendChild(result);

        hint.scrollIntoView();
    }
}

async function update() {
    let input = document.getElementById("input-text");
    let hint = document.getElementById("input-hint");

    input.parentNode.dataset.replicatedValue = input.value;

    let result = await evaluateFend(input.value);

    if (!result.startsWith('Error: ')) {
        hint.innerText = result;
    } else {
        hint.innerText = "";
    }
}

async function load() {
    const { initialise, evaluateFendWithTimeout } = wasm_bindgen;

    await wasm_bindgen('./pkg/fend_wasm_bg.wasm');

    initialise();

    evaluateFendWithTimeout("1 + 2", 500);
    wasmInitialised = true;
};

window.onload = load;
window.onhashchange = load;
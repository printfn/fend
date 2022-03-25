const { initialise, evaluateFendWithTimeout, evaluateFendWithTimeoutMultiple } = wasm_bindgen;

const EVALUATE_KEY = 13;
const NAVIGATE_UP_KEY = 38;
const NAVIGATE_DOWN_KEY = 40;

const VARIABLE_ASSIGN_EXPR = /^\s*([a-z][a-z0-9]*)\s*=/;

let output = document.getElementById("output");
let inputText = document.getElementById("input-text");
let inputHint = document.getElementById("input-hint");
let wasmInitialised = false;
let history = [];
let variables = {};
let navigation = 0;

// allow multiple lines to be entered if shift, ctrl
// or meta is held, otherwise evaluate the expression
async function evaluate(event) {
    if (EVALUATE_KEY == event.keyCode && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();

        let request = document.createElement("p");
        let result = document.createElement("p");

        request.innerText = "> " + inputText.value;

        if (isInputFilled()) {
            history.push(inputText.value);

            if (isInputVariable()) {
                variables[getInputVariable()] = inputText.value;
            }
        }

        navigateEnd();

        inputText.value = "";
        inputHint.innerText = "";

        let results = evaluateFendWithTimeoutMultiple(Object.values(variables).join("\0"), 500).split('\0');

        result.innerText = results[results.length - 1];

        output.appendChild(request);
        output.appendChild(result);

        inputHint.scrollIntoView();
    }
}

function navigate(event) {
    if (NAVIGATE_UP_KEY == event.keyCode || NAVIGATE_DOWN_KEY == event.keyCode) {
        if (navigation > 0) {
            if (NAVIGATE_UP_KEY == event.keyCode) {
                event.preventDefault();

                navigateBackwards();
            }

            else if (NAVIGATE_DOWN_KEY == event.keyCode) {
                event.preventDefault();

                navigateForwards();
            }

        } else if (!isInputFilled() && history.length > 0 && NAVIGATE_UP_KEY == event.keyCode) {
            event.preventDefault();

            navigateBegin();
        }

        if (navigation > 0) {
            navigateSet();
        }

        updateReplicatedText();
        updateHint();
    }
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
    inputText.value = "";
}

function focus() {
    // allow the user to select text for copying and
    // pasting, but if text is deselected (collapsed)
    // refocus the input field
    if (document.activeElement != inputText && document.getSelection().isCollapsed) {
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
    let results = evaluateFendWithTimeoutMultiple(Object.values(variables).join("\0") + '\0' + inputText.value, 500).split('\0');
    let result = results[results.length - 1];

    if (!result.startsWith('Error: ')) {
        inputHint.innerText = result;
    } else {
        inputHint.innerText = "";
    }
}

function isInputFilled() {
    return inputText.value.length > 0;
}

function isInputVariable() {
    return isInputFilled() && VARIABLE_ASSIGN_EXPR.test(inputText.value);
}

function getInputVariable() {
    return inputText.value.match(VARIABLE_ASSIGN_EXPR)[1];
}

async function load() {
    await wasm_bindgen('./pkg/fend_wasm_bg.wasm');

    initialise();

    evaluateFendWithTimeout("1 + 2", 500);
    wasmInitialised = true;

    inputText.addEventListener('input', update);
    inputText.addEventListener('keypress', evaluate);
    inputText.addEventListener('keydown', navigate);
    document.addEventListener('click', focus)
};

window.onload = load;

import fend from 'fend-wasm-nodejs';

const TELEGRAM_BOT_API_TOKEN = process.env.TELEGRAM_BOT_API_TOKEN;

fend.initialise();

/*
Setting a webhook:
TELEGRAM_BOT_API_TOKEN="..."
LAMBDA_URL="..."
curl "https://api.telegram.org/bot${TELEGRAM_BOT_API_TOKEN}/setWebhook" --form-string "url=${LAMBDA_URL}"
*/

async function processInput(input, chatType) {
    if (input == '/start' || input == '/help') {
        return "fend is an arbitrary-precision unit-aware calculator.\n\nYou can send it maths questions like '1+1', 'sin(pi)' or 'sqrt(5)'. In group chats, you'll need to enclose your input in [[double square brackets]] like this: [[1+1]].";
    }

    if (chatType == 'private' || chatType == 'inline') {
        return fend.evaluateFendWithTimeout(input, 500);
    } else if (chatType == 'group' || chatType == 'supergroup' || chatType == 'channel') {
        let response = JSON.parse(fend.substituteInlineFendExpressions(input, 500));
        let result = '';
        let foundCalculation = false;
        for (let part of response) {
            if (part.type == 'fend_output' || part.type == 'fend_error') {
                foundCalculation = true;
            }
            result += part.contents;
        }
        if (!foundCalculation) {
            return null;
        }
        return result;
    }
};

async function processMessage(message) {
    let text = message.text;
    let result = processInput(text, message.chat.type);
    if (result != null && result != '') {
        await postJSON(`https://api.telegram.org/bot${TELEGRAM_BOT_API_TOKEN}/sendChatAction`, {
            chat_id: message.chat.id,
            action: 'typing',
        });
        await postJSON(`https://api.telegram.org/bot${TELEGRAM_BOT_API_TOKEN}/sendMessage`, {
            chat_id: message.chat.id,
            text: result,
            disable_web_page_preview: true,
            disable_notification: true,
        });
    }
};

async function processUpdate(update) {
    console.log('Update: ' + JSON.stringify(update));
    if (update.message && update.message.text) {
        await processMessage(update.message);
    } else if (update.edited_message && update.edited_message.text) {
        await processMessage(update.edited_message);
    } else if (update.inline_query) {
        let input = update.inline_query.query;
        let result = processInput(input, 'inline');
        let results = [];
        if (result != null && result != '') {
            results.push({type: 'article', title: result, id: '1', input_message_content: {message_text: result}});
        }
        await postJSON(`https://api.telegram.org/bot${TELEGRAM_BOT_API_TOKEN}/answerInlineQuery`, {
            inline_query_id: update.inline_query.id,
            results,
        });
    }
};

async function pollUpdates() {
    try {
        var highestOffet = 441392434;
        while (true) {
            console.log('Polling getUpdates (30s)...')
            let updates = await postJSON(`https://api.telegram.org/bot${TELEGRAM_BOT_API_TOKEN}/getUpdates`, {
                timeout: 30,
                offset: highestOffet + 1,
            });
            for (let update of updates) {
                highestOffet = Math.max(highestOffet, update.update_id);
                await processUpdate(update);
            }
        }
    } catch (error) {
        console.log(error);
    }
};

async function postJSON(url, body) {
	let response = await fetch(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json ; charset=UTF-8',
		},
		body: JSON.stringify(body),
	});
	let responseObject = await response.json();
	if (responseObject.ok) {
		return responseObject.result;
	} else {
		const msg = 'Error: ' + JSON.stringify(responseObject);
		console.log(msg);
		throw new Error(msg);
	}
}

export async function handler(event) {
    let update = JSON.parse(event.body);
    try {
        await processUpdate(update);
    } catch (error) {
        console.log(error);
    }
    return { statusCode: 200, body: 'ok' };
}

if (!process.env.AWS_REGION) {
    // running locally
    pollUpdates();
}

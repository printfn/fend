const fend = require('fend-wasm-nodejs');
const https = require("https");

const TELEGRAM_BOT_API_TOKEN = process.env.TELEGRAM_BOT_API_TOKEN;

fend.initialise();

/*
Setting a webhook:
TELEGRAM_BOT_API_TOKEN="..."
LAMBDA_URL="..."
curl "https://api.telegram.org/bot${TELEGRAM_BOT_API_TOKEN}/setWebhook" --form-string "url=${LAMBDA_URL}"
*/

const processInput = (input, chatType) => {
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

const processMessage = async (message) => {
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

const processUpdate = async (update) => {
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

const pollUpdates = async () => {
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

const postJSON = (url, jsonBody, method='POST', headers={}) => {
    return new Promise(function(resolve, reject) {
        let postData = Buffer.from(JSON.stringify(jsonBody), 'utf8');
        let req = https.request(url, {
            method,
            headers: {
                'Content-Type': 'application/json ; charset=UTF-8',
                'Content-Length': postData.length,
                ...headers
            }
        }, res => {
            res.setEncoding("utf8");
            let body = "";
            res.on("data", data => {
                body += data;
            });
            res.on("end", () => {
                let responseObject = JSON.parse(body);
                if (responseObject.ok) {
                    resolve(responseObject.result);
                } else {
                    console.log('Error: ' + JSON.stringify(responseObject));
                    reject(responseObject); // has fields 'description' and 'error_code'
                }
            });
        });
        req.write(postData);
        req.end();
    });
};

if (process.env.AWS_REGION) {
    // we're running in AWS Lambda
    exports.handler = async (event) => {
        let update = JSON.parse(event.body);
        try {
            await processUpdate(update);
        } catch (error) {
            console.log(error);
        }
        return { statusCode: 200, body: 'ok' };
    };
} else {
    // running locally
    pollUpdates();
}

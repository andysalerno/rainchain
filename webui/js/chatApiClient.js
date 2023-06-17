import { getContext } from "./script.js";
import { addNewBotChatBubble, addNewSourceChatBubble, appendTextBotChatBubble } from "./ui.js";
// const URI = "ws://archdesktop.local:5007/api/v1/stream";
const URI = "ws://localhost:5007/api/v1/stream";
export function sendChat(message) {
    const request = {
        message: message
    };
    const json = JSON.stringify(request);
    const socket = getContext().socket;
    socket === null || socket === void 0 ? void 0 : socket.send(json);
}
export function openWebSocketConnection() {
    const socket = new WebSocket(URI);
    socket.onopen = () => {
        console.log("connected via websocket");
    };
    socket.onmessage = (message) => {
        console.log("Got message: " + message);
        const parsed = JSON.parse(message.data);
        handleMessage(parsed);
    };
    socket.onclose = () => {
        addNewBotChatBubble("<connection closed>");
    };
    socket.onerror = () => {
        addNewBotChatBubble("<connection error>");
    };
    getContext().socket = socket;
}
function handleMessage(message) {
    if (message.event == "ToolInfo") {
        addNewSourceChatBubble(message.text);
        return;
    }
    else if (message.message_num === 0) {
        // First message means we need to spawn a new chat bubble:
        addNewBotChatBubble(message.text);
        return;
    }
    if (message.text === undefined) {
        // This is the end of stream message. Nothing to do here.
        return;
    }
    appendTextBotChatBubble(message.text);
}

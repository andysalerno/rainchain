import { sendChat } from "./chatApiClient.js";
function getInputElement() {
    return document.getElementById("user-input");
}
function getChatMessagesSection() {
    return document.getElementById("chat-messages");
}
function getLastBotChatBubble() {
    const all = document.querySelectorAll("div.message.bot");
    const count = all.length;
    return all[count - 1];
}
function getTemplate(id) {
    return document.getElementById(id);
}
function sendBtnClicked() {
    console.log("send clicked!");
    const textContent = getInputElement().value;
    sendChat(textContent);
    console.log("clicked with text: " + textContent);
    // Add chat as new text bubble
    // TODO: add as "pending (grey)" and only light up when confirmed sent over pipe
    addNewUserChatBubble(textContent);
    // Clear text from input
    getInputElement().value = '';
}
function enterKeyDown(e) {
    if (e.key === "Enter") {
        sendBtnClicked();
    }
}
function addNewUserChatBubble(text) {
    text = sanitizeAndPreserveNewlines(text);
    const chatSection = getChatMessagesSection();
    const template = getTemplate("message-user-template");
    const fragment = template.content.cloneNode(true);
    const chatDiv = fragment.querySelector('div');
    chatDiv.innerHTML = text;
    chatSection.appendChild(fragment);
}
function sanitizeAndPreserveNewlines(text) {
    // Create a temporary div and set its textContent to the input.
    let div = document.createElement('div');
    div.textContent = text;
    // Get the sanitized text and replace newline characters with <br /> tags.
    let sanitized = div.textContent;
    sanitized = sanitized.replace(/\n/g, "<br />");
    return sanitized;
}
export function addNewSourceChatBubble(text) {
    text = sanitizeAndPreserveNewlines(text);
    const chatSection = getChatMessagesSection();
    const template = getTemplate("message-sources-template");
    const fragment = template.content.cloneNode(true);
    const span = fragment.querySelector('span');
    span.innerHTML = text;
    chatSection.appendChild(fragment);
}
export function addNewBotChatBubble(text) {
    text = sanitizeAndPreserveNewlines(text);
    const chatSection = getChatMessagesSection();
    const template = getTemplate("message-bot-template");
    const fragment = template.content.cloneNode(true);
    const chatDiv = fragment.querySelector('div');
    chatDiv.innerHTML = text;
    chatSection.appendChild(fragment);
}
export function appendTextBotChatBubble(text) {
    var _a;
    text = sanitizeAndPreserveNewlines(text);
    const bubble = getLastBotChatBubble();
    if (text === '<thinking>' && ((_a = bubble.textContent) === null || _a === void 0 ? void 0 : _a.endsWith('<thinking>'))) {
        return;
    }
    // text = text.replace('\n', '\r\n');
    bubble.innerHTML += text;
}
export function sourceMessageClicked(element) {
    if (element.classList.contains('minimized')) {
        console.log("un-minimizing...");
        element.classList.remove('minimized');
    }
    else {
        console.log("minimizing...");
        element.classList.add('minimized');
    }
}
window.UIFns = { sendBtnClicked, enterKeyDown, sourceMessageClicked };

import { sendChat } from "./chatApiClient.js";

function getInputElement(): HTMLInputElement {
    return document.getElementById("user-input") as HTMLInputElement;
}

function getChatMessagesSection(): HTMLDivElement {
    return document.getElementById("chat-messages") as HTMLInputElement;
}

function getLastBotChatBubble(): HTMLDivElement {
    const all = document.querySelectorAll("div.message.bot");

    const count = all.length;

    return all[count - 1] as HTMLDivElement;
}

function getTemplate(id: string): HTMLTemplateElement {
    return document.getElementById(id) as HTMLTemplateElement;
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

function enterKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
        sendBtnClicked();
    }
}

function addNewUserChatBubble(text: string) {
    text = sanitizeAndPreserveNewlines(text);
    const chatSection = getChatMessagesSection();

    const template = getTemplate("message-user-template");
    const fragment = template.content.cloneNode(true) as DocumentFragment;

    const chatDiv = fragment.querySelector('div') as HTMLDivElement;
    chatDiv.innerHTML = text;

    chatSection.appendChild(fragment);
}

function sanitizeAndPreserveNewlines(text: string): string {
    // Create a temporary div and set its textContent to the input.
    let div = document.createElement('div');
    div.textContent = text;

    // Get the sanitized text and replace newline characters with <br /> tags.
    let sanitized = div.textContent;
    sanitized = sanitized.replace(/\n/g, "<br />");

    return sanitized;
}

export function addNewSourceChatBubble(text: string) {
    text = sanitizeAndPreserveNewlines(text);
    const chatSection = getChatMessagesSection();

    const template = getTemplate("message-sources-template");
    const fragment = template.content.cloneNode(true) as DocumentFragment;

    const span = fragment.querySelector('span') as HTMLSpanElement;
    span.innerHTML = text;

    chatSection.appendChild(fragment);
}

export function addNewBotChatBubble(text: string) {
    text = sanitizeAndPreserveNewlines(text);
    const chatSection = getChatMessagesSection();

    const template = getTemplate("message-bot-template");
    const fragment = template.content.cloneNode(true) as DocumentFragment;

    const chatDiv = fragment.querySelector('div') as HTMLDivElement;
    chatDiv.innerHTML = text;

    chatSection.appendChild(fragment);
}

export function appendTextBotChatBubble(text: string) {
    text = sanitizeAndPreserveNewlines(text);
    const bubble = getLastBotChatBubble();

    if (text === '<thinking>' && bubble.textContent?.endsWith('<thinking>')) {
        return;
    }

    // text = text.replace('\n', '\r\n');

    bubble.innerHTML += text;
}

export function sourceMessageClicked(element: Element) {
    if (element.classList.contains('minimized')) {
        console.log("un-minimizing...");
        element.classList.remove('minimized');
    } else {
        console.log("minimizing...");
        element.classList.add('minimized');
    }
}

(window as any).UIFns = { sendBtnClicked, enterKeyDown, sourceMessageClicked }
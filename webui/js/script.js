import { openWebSocketConnection } from "./chatApiClient.js";
export function getContext() {
    return window.ChatContext;
}
document.addEventListener("DOMContentLoaded", function () {
    window.ChatContext = { socket: undefined };
    openWebSocketConnection();
});

import { openWebSocketConnection } from "./chatApiClient.js";

type Context = {
    socket: WebSocket | null,
};

export function getContext(): Context {
    return (window as any).ChatContext as Context;
}

document.addEventListener("DOMContentLoaded", function () {
    (window as any).ChatContext = { socket: undefined };
    openWebSocketConnection();
});

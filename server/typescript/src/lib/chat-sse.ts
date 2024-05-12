import { ReadyStateEvent, SSE, SSEvent } from "sse.js";
import { renderMessage, updateMessage } from "./chat-models";

export function chatSse() {
    const form = document.getElementById('chat-form') as HTMLFormElement;
    const textarea = document.getElementById('chat-input') as HTMLTextAreaElement;

    const onSubmit = (event: Event) => {
        event.preventDefault();
        textarea.disabled = true;

        let name = new FormData(form).values().next().value;
        let message = textarea.value;
        renderMessage(message, name);

        textarea.value = '';

        const url = form.action;
        const source = new SSE(url, {
            withCredentials: true,
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            payload: `message=${encodeURIComponent(message)}`,
            start: false,
        });

        let messageId = 0;
        let response = '';

        source.onreadystatechange = (e: ReadyStateEvent) => {
            if (e.readyState === 1) {
                messageId = renderMessage('');
            } else if (e.readyState === 2) {
                source.close();
                textarea.disabled = false;
                textarea.focus();
            }
        }

        source.onmessage = (event: SSEvent) => {
            response += event.data;
            updateMessage(messageId, response)
        }

        source.stream();
    };

    const onEnter = (event: KeyboardEvent) => {
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            form?.dispatchEvent(new Event('submit'));
        }
    }

    form?.addEventListener('submit', onSubmit);
    textarea?.addEventListener('keydown', onEnter);
}
import { ReadyStateEvent, SSE, SSEvent } from "sse.js";

function submitListener() {
    const form = document.getElementById('finalize-project-button') as HTMLFormElement;
    if (!form) return;

    form.addEventListener('submit', async (event: SubmitEvent) => {
        event.preventDefault();
        let url = (event.target as HTMLFormElement).action;

        let source = new SSE(url, {
            withCredentials: true,
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            start: false,
        });

        source.onreadystatechange = (event: ReadyStateEvent) => {
            if (event.readyState === 2) {
                location.reload();
            }
        }
        source.onmessage = (event: SSEvent) => {
            if (event.data) createOverlay(event.data);
        }

        source.stream();
    });
}

function load() {
    submitListener();
}

htmx.onLoad(load);

function createOverlay(message: string) {
    const existingOverlay = document.getElementById('body-overlay');
    if (existingOverlay) {
        existingOverlay.remove();
    }

    const overlay = document.createElement('div');
    overlay.id = 'body-overlay';
    overlay.innerHTML = `
        <div class="overlay">
            <div class="overlay-content">
                <div class="w-8 h-8 animate-spin">
                    <svg width="100%" height="100%" viewBox="0 0 14 14" xmlns="http://www.w3.org/2000/svg">
                        <g fill="none" fill-rule="evenodd">
                            <circle cx="7" cy="7" r="6" stroke="#fff" stroke-opacity="0.4" stroke-width="2"/>
                            <path fill="#fff" fill-opacity=".8" fill-rule="nonzero" d="M7 0a7 7 0 0 1 7 7h-2a5 5 0 0 0-5-5V0z"/>
                        </g>
                    </svg>
                </div>
                <p class="font-bold text-center">${message.replaceAll('\n', '<br>')}</p>
            </div>
        </div>`;
    document.body.appendChild(overlay);
    return overlay;
}
import { OnReadystatechange, ReadyStateEvent, SSE, SSEvent } from "sse.js";

function submitListener() {
    const form = document.getElementById('crawler-form') as HTMLFormElement;
    const labels = form.getElementsByTagName('label');
    const inputs = form.getElementsByTagName('input');

    const outputContainer = document.getElementById('crawler-output-container') as HTMLDivElement;
    const output = document.getElementById('crawler-output') as HTMLParagraphElement;

    form.addEventListener('submit', async (event: SubmitEvent) => {
        event.preventDefault();

        // x-www-form-urlencoded form data
        const formData = [];
        for (const pair of new FormData(form).entries()) {
            if (pair[0] === 'toggle-depth') continue;
            const key = encodeURIComponent(pair[0]);
            const value = encodeURIComponent(pair[1] as string | number | boolean);
            formData.push(`${key}=${value}`);
        }

        for (const input of inputs) {
            input.disabled = true;
        }
        for (const label of labels) {
            label.classList.add('disabled');
        }

        let url = (event.target as HTMLFormElement).action;
        let source = new SSE(url, {
            withCredentials: true,
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            payload: formData.join('&'),
            start: false,
        });

        source.onreadystatechange = (event: ReadyStateEvent) => {
            console.log(event);
            if (event.readyState === 1) {
                outputContainer.classList.add('flex');
                outputContainer.classList.remove('hidden');
            } else if (event.readyState === 2) {
                location.reload();
            }
        }
        source.onmessage = (event: SSEvent) => {
            console.log(event.data);
            output.textContent = event.data;
        }

        source.stream();

    });
}

function checkbox() {
    const checkbox = document.getElementById('toggle-depth') as HTMLInputElement;
    const label = document.getElementById('depth-label') as HTMLLabelElement;
    const depth = document.getElementById('depth') as HTMLInputElement;

    checkbox.addEventListener('change', () => {
        depth.disabled = !checkbox.checked;
        label.classList.toggle('disabled', !checkbox.checked);
    });
}

function load() {
    checkbox();
    submitListener();
}

htmx.onLoad(load);
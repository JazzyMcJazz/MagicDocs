let init = false;
let displayingSpinner = false;

function browserSync() {
    const sse = new EventSource('/browser-sync');

    sse.onopen = () => {
        if (init) {
            location.reload();
        } else {
            init = true;
        }
    }

    sse.onerror = () => {
        sse.close();
        console.error = () => {};
        displaySpinner();
        setTimeout(browserSync, 200);
    }

}

let spinnerSvg = '';
async function setSpinnerSvg() {
}

function displaySpinner() {
    if (displayingSpinner) return;
    displayingSpinner = true;
    const spinner = document.createElement('div');
    spinner.classList.add('browser-sync-spinner');
    spinner.innerHTML = `
        <svg width="100%" height="100%" viewBox="0 0 14 14"">
            <g fill="none" fill-rule="evenodd">
                <circle cx="7" cy="7" r="6" stroke="#fff" stroke-opacity=".2" stroke-width="2"/>
                <path fill="#fff" fill-opacity=".3" fill-rule="nonzero" d="M7 0a7 7 0 0 1 7 7h-2a5 5 0 0 0-5-5V0z"/>
            </g>
        </svg>
    `;
    document.body.appendChild(spinner);
}

htmx.onLoad(browserSync);
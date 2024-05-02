
function load() {
    const checkbox = document.getElementById('toggle-depth') as HTMLInputElement;
    const label = document.getElementById('depth-label') as HTMLLabelElement;
    const depth = document.getElementById('depth') as HTMLInputElement;

    checkbox.addEventListener('change', () => {
        depth.disabled = !checkbox.checked;
        label.classList.toggle('disabled', !checkbox.checked);
    });

}

htmx.onLoad(load);
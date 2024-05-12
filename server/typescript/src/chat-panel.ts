let panel: HTMLElement;
let header: HTMLElement;
let expandButton: HTMLButtonElement;
let openButton: HTMLButtonElement;
let chatOptionsButton: HTMLButtonElement;

htmx.onLoad(() => {
    panel = document.getElementById('chat-panel') as HTMLElement;
    header = document.getElementById('chat-header') as HTMLElement;
    expandButton = document.getElementById('chat-expand') as HTMLButtonElement;
    openButton = document.getElementById('chat-open') as HTMLButtonElement;
    chatOptionsButton = document.getElementById('chat-options') as HTMLButtonElement;

    expandButton?.addEventListener('click', onExpandClick);
    openButton?.addEventListener('click', onToggleOpenClick);
});

function onToggleOpenClick() {
    const open = panel.classList.toggle('open');
    if (open) {
        chatOptionsButton.classList.remove('hidden');
    } else {
        chatOptionsButton.classList.add('hidden');
    }
}

function onExpandClick() {
    const expanded = panel.classList.toggle('expanded');
    const open = panel.classList.contains('open');
    if (expanded) {
        header.classList.remove('rounded-t-lg');
        panel.classList.remove('-bottom-[calc(24rem-3.5rem)]');
        chatOptionsButton.classList.remove('hidden');
    } else {
        header.classList.add('rounded-t-lg');
        panel.classList.add('-bottom-[calc(24rem-3.5rem)]');
    }

    if (!expanded && !open) {
        chatOptionsButton.classList.add('hidden');
    }
}

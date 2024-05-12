
export function toggleOpenOnClick() {
    const panel = document.getElementById('chat-panel') as HTMLElement;
    const openButton = document.getElementById('chat-open') as HTMLButtonElement;
    const chatOptionsButton = document.getElementById('chat-options') as HTMLButtonElement;

    const callback = () => {
        const expanded = panel.classList.contains('expanded');
        if (expanded) return

        const open = panel.classList.toggle('open');
        if (open) {
            chatOptionsButton.classList.remove('hidden');
        } else {
            chatOptionsButton.classList.add('hidden');
        }
    }

    openButton?.addEventListener('click', callback);
}

export function toggleExpandedOnClick() {
    const panel = document.getElementById('chat-panel') as HTMLElement;
    const header = document.getElementById('chat-header') as HTMLElement;
    const expandButton = document.getElementById('chat-expand') as HTMLButtonElement;
    const chatOptionsButton = document.getElementById('chat-options') as HTMLButtonElement;

    const callback = () => {
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

    expandButton?.addEventListener('click', callback);
}
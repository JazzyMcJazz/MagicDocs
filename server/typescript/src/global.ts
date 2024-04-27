
let userMenu: Element | null = null;

const clickOutside = (event: MouseEvent) => {
    if (userMenu && !userMenu.contains(event.target as Node)) {
        userMenu.classList.add('hidden');
        document.removeEventListener('click', clickOutside);
    }
};

function toggleUserMenu() {
    if (!userMenu) return;
    const isHidden = userMenu.classList.toggle('hidden');
    if (isHidden) {
        document.removeEventListener('click', clickOutside);
    } else {
        setTimeout(() => document.addEventListener('click', clickOutside));
    }
}

function preventSamePage(e: CustomEvent<HtmxRequestEvent>) {
    const { verb, path } = e.detail.requestConfig;
    if (verb !== 'get') return;

    const currentUrl = new URL(window.location.href);
    if (path === currentUrl.pathname) {
        e.preventDefault();
    }
}

htmx.onLoad(() => {
    userMenu = htmx.find('#nav-user-menu');
    htmx.find('#nav-user-menu-btn')?.addEventListener('click', toggleUserMenu);
    htmx.find('body')?.addEventListener('htmx:beforeRequest', e => preventSamePage(e as CustomEvent<HtmxRequestEvent>));
});

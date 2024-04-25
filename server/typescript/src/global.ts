
let userMenu: HTMLElement | null = null;

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

htmx.onLoad(() => {
    document.getElementById('nav-user-menu-btn')?.addEventListener('click', toggleUserMenu);
    userMenu = document.getElementById('nav-user-menu');
});
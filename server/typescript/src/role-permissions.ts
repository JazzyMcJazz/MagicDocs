type RolePermission = { read?: boolean, write?: boolean, delete?: boolean };
type RoleState = { [key:string]: RolePermission };

const submitListener = async (event: Event, initState: RoleState) => {
    event.preventDefault();

    const form = event.target as HTMLFormElement;
    const url = form.action;
    const json = getRoleState(form);

    for (const projectId in json) {
        if (json[projectId]?.read === initState[projectId]?.read) {
            delete json[projectId].read;
        }
        if (json[projectId]?.write === initState[projectId]?.write) {
            delete json[projectId].write;
        }
        if (json[projectId]?.delete === initState[projectId]?.delete) {
            delete json[projectId].delete;
        }
        if (Object.keys(json[projectId]).length === 0) {
            delete json[projectId];
        }
    }

    await fetch(url, {
        method: 'POST',
        credentials: 'same-origin',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ data: json }),
    });

    location.reload();
};

const getRoleState = (form: HTMLFormElement) => {
    let inputs = form.querySelectorAll('input[type="checkbox"]') as NodeListOf<HTMLInputElement>;

    const state: { [key:string]: RolePermission } = {};
    inputs.forEach((input) => {
        const [projectId, permission] = input.name.split('-') as [string, 'read' | 'write' | 'delete'];
        if (!state[projectId]) {
            state[projectId] = { read: false, write: false, delete: false };
        }
        if (input.checked) {
            state[projectId][permission] = true;
        }
    });

    return state;
}

htmx.onLoad(() => {
    const form = document.getElementById('role-permissions-form') as HTMLFormElement;
    const initState: { [key:string]: RolePermission } = getRoleState(form);
    form?.addEventListener('submit', (e) => submitListener(e, initState));
});
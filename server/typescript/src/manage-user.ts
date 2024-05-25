type UserPermission = { read?: boolean, write?: boolean, delete?: boolean };
type UserRoleState = { id: string, name: string };
type UserState = {
    roles: Array<UserRoleState>,
    permissions: { [key:string]: UserPermission },
};

const initState: UserState = { roles: [], permissions: {} };

const getUserState = () => {
    let form = document.getElementById('user-permissions-form') as HTMLFormElement;
    let inputs = form.querySelectorAll('input[type="checkbox"]') as NodeListOf<HTMLInputElement>;

    const permissions: { [key:string]: UserPermission } = {};
    inputs.forEach((input) => {
        const [projectId, permission] = input.name.split('-') as [string, 'read' | 'write' | 'delete'];
        if (!permissions[projectId]) {
            permissions[projectId] = { read: false, write: false, delete: false };
        }
        if (input.checked) {
            permissions[projectId][permission] = true;
        }
    });

    const assignedRoles = document.getElementById('assigned-roles-select') as HTMLSelectElement;
    const roles =  Array.from(assignedRoles.options).map(option => ({ id: option.value.split('¤')[0], name: option.value.split('¤')[1] }));

    return { roles, permissions };
}

const getRoleSelects = () => {
    const availableRoles = document.getElementById('available-roles-select') as HTMLSelectElement;
    const assignedRoles = document.getElementById('assigned-roles-select') as HTMLSelectElement;
    return { availableRoles, assignedRoles };
}

const getSelectedRole = (selectElement: HTMLSelectElement) => {
    return Array.from(selectElement.selectedOptions);
}

const moveSelectedRoles = (from: HTMLSelectElement, to: HTMLSelectElement) => {
    const selected = getSelectedRole(from);

    const moved: Array<string> = [];
    selected.forEach(option => {
        moved.push(option.value);
        to.appendChild(option);
    });

    return moved;
}

const assignRole = () => {
    const { availableRoles, assignedRoles } = getRoleSelects();
    moveSelectedRoles(availableRoles, assignedRoles);
}

const removeRole = () => {
    const { availableRoles, assignedRoles } = getRoleSelects();
    moveSelectedRoles(assignedRoles, availableRoles);
}

const updateUser = async (e: SubmitEvent) => {
    e.preventDefault();

    const state = getUserState();

    const roles_to_assign = state.roles
        .filter(role => !initState.roles.find(r => r.id === role.id));

    const roles_to_revoke = initState.roles
        .filter(role => !state.roles.find(r => r.id === role.id));

    const url = (e.target as HTMLFormElement).action;

    const permissions = {...state.permissions};
    for (const projectId in permissions) {
        if (permissions[projectId]?.read === initState.permissions[projectId]?.read) {
            delete permissions[projectId].read;
        }
        if (permissions[projectId]?.write === initState.permissions[projectId]?.write) {
            delete permissions[projectId].write;
        }
        if (permissions[projectId]?.delete === initState.permissions[projectId]?.delete) {
            delete permissions[projectId].delete;
        }
        if (Object.keys(permissions[projectId]).length === 0) {
            delete permissions[projectId];
        }
    }

    await fetch(url, {
        credentials: 'same-origin',
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            permissions,
            roles_to_assign,
            roles_to_revoke,
        })
    });

    location.reload();
}

const load = () => {
    const assignRoleButton = document.getElementById('assign-role-btn');
    const removeRoleButton = document.getElementById('remove-role-btn');
    const saveForm = document.getElementById('save-user-form');

    let state = getUserState();
    initState.roles = state.roles;
    initState.permissions = state.permissions;

    assignRoleButton?.addEventListener('click', assignRole);
    removeRoleButton?.addEventListener('click', removeRole);
    saveForm?.addEventListener('submit', updateUser);

}

htmx.onLoad(load);
import { test as teardown } from '@playwright/test';
import files from '../files';

teardown.describe(() => {
    teardown.use({ storageState: files.userFile });

    teardown('logout user', async ({ page }) => {
        await page.goto('/');
        await page.click('#nav-user-menu-btn');
        const res = page.waitForResponse('**/logout');
        await page.click('a[hx-post="/logout"]', { delay: 100 }); // wait for htmx
        await res;
    });
});

teardown.describe(() => {
    teardown.use({ storageState: files.adminFile });

    teardown('logout admin', async ({ page }) => {
        await page.goto('/');
        await page.click('#nav-user-menu-btn');
        const res = page.waitForResponse('**/logout');
        await page.click('a[hx-post="/logout"]', { delay: 100 }); // wait for htmx
        await res;
    });
});

teardown.describe(() => {
    teardown.use({ storageState: files.superAdminFile });

    teardown('logout super admin', async ({ page }) => {

        await page.goto('/');
        await page.click('#nav-user-menu-btn');
        const res = page.waitForResponse('**/logout');
        await page.click('a[hx-post="/logout"]', { delay: 100 }); // wait for htmx
        await res;
    });
});
import test, { expect } from "@playwright/test";
import files from "../files";

test.describe('user', () => {
    test.use({ storageState: files.userFile });

    test('cannot see any projects', async ({ page }) => {
       // Start from the home page
       await page.goto('/');

       // Check that the new project button is not visible
       const newProjectButton = page.getByText('New Project');
       await expect(newProjectButton).not.toBeVisible();

       const sideBarItem = page.locator('#sidebar');
       await expect(sideBarItem).toHaveText('');
    });

    test('cannot visit project creation page', async ({ page }) => {
       const result = await page.goto('/projects/new');
       expect(result.status()).toBe(403);
    });

    test('cannot visit project details page', async ({ page }) => {
       const result = await page.goto('/projects/1');
       expect(result.status()).toBe(403);
    });

    test('cannot submit project creation form', async ({ request }) => {
       const result = await request.post('/projects', { form: { "project-name": 'Illegal Project', description: "MUAHAHA" } });
       expect(result.status()).toBe(403);
    });
});

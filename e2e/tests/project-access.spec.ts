import { test, expect } from '@playwright/test';
import files from '../files';

test.describe('as admin', () => {
   test.use({ storageState: files.adminFile });

   test('can create new project', async ({ page }, workerinfo) => {
      const projectName = `${workerinfo.project.name} - ${Date.now()}`;

      // Start from the home page
      await page.goto('/');

      // Click on the new project button
      const newProjectButton = page.getByText('New Project');
      await expect(newProjectButton).toBeVisible();
      await newProjectButton.click();

      // Check that the page title is correct
      await expect(page).toHaveTitle(/Magic Docs - New Project/);

      // Fill the form and submit
      await page.getByLabel('Project Name').fill(projectName);
      await page.getByLabel('Description').fill('Test Description');
      await page.getByRole('button', { name: 'Create Project' }).click();

      // Check that we are redirected to the project page
      await expect(page).toHaveTitle(`Magic Docs - ${projectName}`);
      const projectTitle = page.getByRole('heading', { name: projectName });
      await expect(projectTitle).toBeVisible();

      // Check that the project is visible in the sidebar
      const sideBarItem = page.getByTitle(projectName);
      await expect(sideBarItem).toBeVisible();
      await expect(sideBarItem).toHaveClass(/active-project/);
   });
});

test.describe('as user', () => {
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

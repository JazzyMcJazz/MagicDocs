import { test, expect } from '@playwright/test';
import files from '../files';

test.describe('admin', () => {
   test.use({ storageState: files.adminFile });
   let projectName: string;

   test('can create new project', async ({ page }, workerinfo) => {
      projectName = `${workerinfo.project.name} - ${Date.now()}`;

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

   test('can create a new document and get access', async ({ page }) => {
      await page.goto('/');

      await page.getByTitle(projectName).click();
      await page.getByRole('link', { name: 'New Document' }).click();

      // Can see the new document page
      await expect(page.getByRole('heading', { name: 'New Document' })).toBeVisible();

      // Can fill the form and submit
      const input = page.getByPlaceholder('Document Title');
      await expect(input).toBeEditable();
      await input.click();
      await input.fill('Test Document');
      const editor = page.locator('div[role="textbox"]');
      await expect(editor).toBeVisible();
      await editor.pressSequentially('## This is a Test Document\n\n```js\nconsole.log("Hello World");\n```')
      await page.getByRole('button', { name: 'Save' }).click();

      // Is redirected to the document page
      await expect(page.getByRole('heading', { name: 'Test Document', exact: true })).toBeVisible();

      // Go back to the project page
      await page.getByTitle(projectName).click();

      // Can see the document in the sidebar
      await expect(page.getByRole('link', { name: 'Test Document', exact: true })).toBeVisible();

      // Can access the document page from the sidebar
      await page.getByRole('link', { name: 'Test Document', exact: true }).click();
      await expect(page.getByRole('heading', { name: 'Test Document', exact: true })).toBeVisible();
   });
});

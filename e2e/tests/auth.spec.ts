import { test, expect } from '@playwright/test';

const TEST_USER_USERNAME = process.env.TEST_USER_USERNAME;
const TEST_USER_PASSWORD = process.env.TEST_USER_PASSWORD;

test.beforeEach(async ({ page }) => {
   await page.goto('/');
   await expect(page).toHaveTitle(/Sign in to Magic Docs/);
   await page.fill('input[name="username"]', TEST_USER_USERNAME);
   await page.fill('input[name="password"]', TEST_USER_PASSWORD);
   await page.click('input[type="submit"]');
});

test.afterEach(async ({ page }) => {
   await page.goto('/');
   await page.click('#nav-user-menu-btn');
   await page.click('a[hx-post="/logout"]');
   await expect(page).toHaveTitle(/Sign in to Magic Docs/);

});

test('is logged ind', async ({ page }) => {
   await page.goto('/');
   await expect(page).toHaveTitle(/Magic Docs/);
});


// test('get started link', async ({ page }) => {
//    await page.goto('https://playwright.dev/');

//    // Click the get started link.
//    await page.getByRole('link', { name: 'Get started' }).click();

//    // Expects page to have a heading with the name of Installation.
//    await expect(page.getByRole('heading', { name: 'Installation' })).toBeVisible();
// });

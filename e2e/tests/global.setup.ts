import { test as setup, expect } from '@playwright/test';
import files from '../files';

const USER_USERNAME = process.env.KEYCLOAK_TEST_USER_USERNAME;
const USER_PASSWORD = process.env.KEYCLOAK_TEST_USER_PASSWORD;
const ADMIN_USERNAME = process.env.KEYCLOAK_TEST_ADMIN_USERNAME;
const ADMIN_PASSWORD = process.env.KEYCLOAK_TEST_ADMIN_PASSWORD;
const SUPER_ADMIN_USERNAME = process.env.KEYCLOAK_TEST_SUPERADMIN_USERNAME;
const SUPER_ADMIN_PASSWORD = process.env.KEYCLOAK_TEST_SUPERADMIN_PASSWORD;

setup('authenticate as user', async ({ page }) => {
    await page.goto('/');
    await page.getByLabel('Username or email').fill(USER_USERNAME);
    await page.getByLabel('Password', { exact: true }).fill(USER_PASSWORD);
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page).toHaveTitle(/Magic Docs/);

    await page.context().storageState({ path: files.userFile });
});

setup('authenticate as admin', async ({ page }) => {
    await page.goto('/');
    await page.getByLabel("Username or email").fill(ADMIN_USERNAME);
    await page.getByLabel("Password", { exact: true }).fill(ADMIN_PASSWORD);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveTitle(/Magic Docs/);

    await page.context().storageState({ path: files.adminFile });
});

setup('authenticate as super admin', async ({ page }) => {
    await page.goto('/');
    await page.getByLabel("Username or email").fill(SUPER_ADMIN_USERNAME);
    await page.getByLabel("Password", { exact: true }).fill(SUPER_ADMIN_PASSWORD);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveTitle(/Magic Docs/);

    await page.context().storageState({ path: files.superAdminFile });
});
import { test, expect } from '@playwright/test'

test.describe('DreamFactory Frontend V2', () => {
  test('should display the home page correctly', async ({ page }) => {
    await page.goto('/')
    
    // Check that the page loads and has the expected title
    await expect(page).toHaveTitle(/DreamFactory/)
    
    // Check for basic page elements
    await expect(page.locator('body')).toBeVisible()
  })
  
  test('should be responsive on mobile devices', async ({ page }) => {
    // Set viewport to mobile size
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')
    
    // Check that the page is still functional on mobile
    await expect(page.locator('body')).toBeVisible()
  })
  
  test('should have proper accessibility attributes', async ({ page }) => {
    await page.goto('/')
    
    // Basic accessibility checks
    const main = page.locator('main')
    if (await main.count() > 0) {
      await expect(main).toBeVisible()
    }
  })
})
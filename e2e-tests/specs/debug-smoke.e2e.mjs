import assert from 'node:assert/strict';

describe('Tauri Desktop Smoke - Debug', () => {
  it('debug: check what page is loaded', async () => {
    // Wait for app to initialize
    await browser.pause(10000);
    
    // Get current URL
    const currentUrl = await browser.getUrl();
    console.log('Current URL:', currentUrl);
    
    // Get page title
    const title = await browser.getTitle();
    console.log('Page title:', title);
    
    // Get page source to see what's rendered
    const source = await browser.getPageSource();
    console.log('Page source (first 1000 chars):', source.substring(0, 1000));
    
    // Try to find any h1 element
    const allH1 = await $$('h1');
    console.log('Number of h1 elements found:', allH1.length);
    
    // Try to find any content
    const bodyText = await $('body').getText();
    console.log('Body text (first 500 chars):', bodyText.substring(0, 500));
    
    // Check if there are any errors in the page
    // Try to find the login page elements
    const signInBtn = await $('//button[contains(., "Sign in")]');
    const isSignInDisplayed = await signInBtn.isDisplayed().catch(() => false);
    console.log('Sign in button displayed:', isSignInDisplayed);
    
    assert.ok(true, 'Debug test completed');
  });
});

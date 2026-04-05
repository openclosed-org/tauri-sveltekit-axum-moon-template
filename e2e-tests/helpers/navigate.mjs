const APP_ORIGIN = process.env.TAURI_E2E_BASE_URL || 'http://tauri.localhost';

function toAppUrl(path) {
  return new URL(path, APP_ORIGIN).toString();
}

export async function navigateTo(path) {
  await browser.url(toAppUrl(path));

  await browser.waitUntil(
    async () => {
      try {
        const currentUrl = await browser.getUrl();
        if (currentUrl === 'about:blank') {
          return false;
        }

        const readyState = await browser.execute(() => document.readyState);
        return readyState === 'interactive' || readyState === 'complete';
      } catch {
        return false;
      }
    },
    {
      timeout: 15000,
      interval: 250,
      timeoutMsg: `Navigation to ${path} did not complete in time`,
    },
  );
}

export async function isLoginPageVisible() {
  const body = await getBodyText();
  return body.includes('Sign in with Google') || body.includes('Welcome back');
}

export async function getBodyText() {
  try {
    return await $('body').getText();
  } catch {
    return '';
  }
}

export async function waitForText(text, timeout = 10000) {
  await browser.waitUntil(
    async () => {
      const body = await getBodyText();
      return body.includes(text);
    },
    {
      timeout,
      interval: 250,
      timeoutMsg: `Text not found in page body: ${text}`,
    },
  );
}

export async function waitForAnyText(texts, timeout = 10000) {
  await browser.waitUntil(
    async () => {
      const body = await getBodyText();
      return texts.some((text) => body.includes(text));
    },
    {
      timeout,
      interval: 250,
      timeoutMsg: `None of the expected texts were found: ${texts.join(', ')}`,
    },
  );
}

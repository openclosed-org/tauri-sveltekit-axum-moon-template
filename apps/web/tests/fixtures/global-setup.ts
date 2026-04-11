import type { FullConfig } from '@playwright/test';
import { ensureWebE2EPreflight } from './runtime';

export default async function globalSetup(_config: FullConfig): Promise<void> {
  await ensureWebE2EPreflight();
}

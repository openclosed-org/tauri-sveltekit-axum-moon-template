/**
 * Verify Counter Delivery — ensure the counter reference chain has an executable
 * delivery admission path instead of relying on drift-prone prose.
 *
 * Usage:
 *   bun run scripts/verify-counter-delivery.ts [--mode warn|strict]
 */

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { runSync } from './lib/spawn.ts';

type Mode = 'warn' | 'strict';

function parseMode(argv: string[]): Mode {
  let mode: Mode = 'warn';
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--mode') {
      const value = argv[index + 1];
      if (value === 'warn' || value === 'strict') {
        mode = value;
        index += 1;
        continue;
      }
    }

    console.error(`Unknown argument: ${arg}`);
    console.error('Usage: bun run scripts/verify-counter-delivery.ts [--mode warn|strict]');
    process.exit(1);
  }
  return mode;
}

function read(relativePath: string): string {
  return readFileSync(path.join(process.cwd(), relativePath), 'utf-8');
}

function requireFile(relativePath: string, failures: string[], reason: string): void {
  if (!existsSync(path.join(process.cwd(), relativePath))) {
    failures.push(`${relativePath}: ${reason}`);
  }
}

function requireContent(
  relativePath: string,
  rule: RegExp,
  failures: string[],
  reason: string,
): void {
  const absolutePath = path.join(process.cwd(), relativePath);
  if (!existsSync(absolutePath)) {
    failures.push(`${relativePath}: missing file`);
    return;
  }

  const content = readFileSync(absolutePath, 'utf-8');
  if (!rule.test(content)) {
    failures.push(`${relativePath}: ${reason}`);
  }
}

function main(): number {
  const mode = parseMode(process.argv.slice(2));
  const failures: string[] = [];

  const requiredFiles = [
    'infra/security/sops/scripts/verify-counter-shared-db.sh',
    'infra/security/sops/dev/counter-shared-db.enc.yaml',
    'infra/k3s/overlays/dev/outbox-relay-worker/kustomization.yaml',
    'infra/k3s/overlays/dev/projector-worker/kustomization.yaml',
    // Staging overlays — multi-env promotion
    'infra/k3s/overlays/staging/outbox-relay-worker/kustomization.yaml',
    'infra/k3s/overlays/staging/projector-worker/kustomization.yaml',
    'infra/gitops/flux/apps/staging-outbox-relay-worker.yaml',
    'infra/gitops/flux/apps/staging-projector-worker.yaml',
    'infra/gitops/flux/apps/outbox-relay-worker.yaml',
    'infra/gitops/flux/apps/projector-worker.yaml',
    'docs/operations/counter-service-reference-chain.md',
    'ops/runbooks/counter-delivery.md',
    // Platform model drift — deployables must exist for the counter chain
    'platform/model/deployables/web-bff.yaml',
    'platform/model/deployables/outbox-relay-worker.yaml',
    'platform/model/deployables/projector-worker.yaml',
    'platform/model/deployables/counter-service.yaml',
    'platform/model/services/counter-service.yaml',
    'platform/model/state/ownership-map.yaml',
  ];

  for (const relativePath of requiredFiles) {
    requireFile(relativePath, failures, 'required counter delivery artifact missing');
  }

  requireContent(
    'infra/k3s/overlays/dev/outbox-relay-worker/kustomization.yaml',
    /counter-shared-db-secrets/,
    failures,
    'outbox relay overlay must consume counter shared DB secret',
  );
  requireContent(
    'infra/k3s/overlays/dev/projector-worker/kustomization.yaml',
    /counter-shared-db-secrets/,
    failures,
    'projector overlay must consume counter shared DB secret',
  );
  requireContent(
    'infra/k3s/overlays/dev/outbox-relay-worker/kustomization.yaml',
    /path:\s*\/spec\/replicas[\s\S]*value:\s*1/,
    failures,
    'outbox relay overlay must keep replicas patched to 1',
  );
  requireContent(
    'infra/k3s/overlays/dev/projector-worker/kustomization.yaml',
    /path:\s*\/spec\/replicas[\s\S]*value:\s*1/,
    failures,
    'projector overlay must keep replicas patched to 1',
  );
  requireContent(
    'infra/gitops/flux/apps/outbox-relay-worker.yaml',
    /path:\s*\.\/infra\/k3s\/overlays\/dev\/outbox-relay-worker/,
    failures,
    'outbox relay Flux app must point at its dev overlay',
  );
  requireContent(
    'infra/gitops/flux/apps/projector-worker.yaml',
    /path:\s*\.\/infra\/k3s\/overlays\/dev\/projector-worker/,
    failures,
    'projector Flux app must point at its dev overlay',
  );

  // --- Staging promotion checks ---
  requireContent(
    'infra/gitops/flux/apps/staging-outbox-relay-worker.yaml',
    /path:\s*\.\/infra\/k3s\/overlays\/staging\/outbox-relay-worker/,
    failures,
    'staging outbox relay Flux app must point at its staging overlay',
  );
  requireContent(
    'infra/gitops/flux/apps/staging-projector-worker.yaml',
    /path:\s*\.\/infra\/k3s\/overlays\/staging\/projector-worker/,
    failures,
    'staging projector Flux app must point at its staging overlay',
  );
  requireContent(
    'infra/gitops/flux/apps/staging-outbox-relay-worker.yaml',
    /ENV:\s*staging/,
    failures,
    'staging outbox relay Flux app must substitute ENV=staging',
  );
  requireContent(
    'infra/k3s/overlays/staging/outbox-relay-worker/kustomization.yaml',
    /counter-shared-db-staging/,
    failures,
    'staging outbox relay overlay must reference staging counter DB secret',
  );
  requireContent(
    'docs/operations/counter-service-reference-chain.md',
    /verify-counter-delivery|counter delivery gate/i,
    failures,
    'counter reference chain must mention executable delivery admission',
  );
  requireContent(
    'ops/runbooks/counter-delivery.md',
    /just verify-counter-delivery strict|counter-shared-db/i,
    failures,
    'counter delivery runbook must document executable admission and shared DB checks',
  );

  // --- Drift checks: platform model alignment ---
  requireContent(
    'platform/model/state/ownership-map.yaml',
    /entity:\s*counter/,
    failures,
    'ownership-map must declare counter entity owner',
  );
  requireContent(
    'platform/model/deployables/web-bff.yaml',
    /counter-service/,
    failures,
    'web-bff deployable must list counter-service',
  );
  requireContent(
    'platform/model/deployables/counter-service.yaml',
    /current_status:\s*embedded[\s\S]*target_status:\s*independent[\s\S]*embedded_in:\s*\n\s*-\s*web-bff/,
    failures,
    'counter-service deployable must distinguish current embedded status from independent target status',
  );
  requireContent(
    'platform/model/deployables/projector-worker.yaml',
    /runtime_profile:\s*async-projection/,
    failures,
    'projector-worker deployable must declare async-projection profile',
  );

  // --- Rollback check: runbook must document rollback procedure ---
  requireContent(
    'ops/runbooks/counter-delivery.md',
    /rollback/i,
    failures,
    'counter delivery runbook must document rollback procedure',
  );

  const sharedDbCheck = runSync('bash', ['infra/security/sops/scripts/verify-counter-shared-db.sh', 'dev']);
  if (!sharedDbCheck.success) {
    failures.push(
      `counter shared DB secret verification failed: ${sharedDbCheck.error || sharedDbCheck.output || 'unknown error'}`,
    );
  }

  if (failures.length === 0) {
    console.log('Counter delivery verification passed (promotion + drift + rollback)');
    return 0;
  }

  for (const failure of failures) {
    console.log(`WARN ${failure}`);
  }

  return mode === 'strict' ? 1 : 0;
}

process.exit(main());

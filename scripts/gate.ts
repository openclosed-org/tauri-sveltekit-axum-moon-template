import process from 'node:process';
import { run } from './lib/spawn.ts';

type GateMode = 'warn' | 'strict';
type GateName = 'local' | 'prepush' | 'ci' | 'release';

interface GateCommand {
  label: string;
  command: string;
  args: string[];
}

const gateDefinitions: Record<GateName, GateCommand[]> = {
  local: [
    {
      label: 'toolchain doctor',
      command: 'just',
      args: ['doctor'],
    },
    {
      label: 'format check',
      command: 'just',
      args: ['fmt'],
    },
    {
      label: 'lint',
      command: 'just',
      args: ['lint'],
    },
  ],
  prepush: [
    {
      label: 'existence validation',
      command: 'just',
      args: ['gate-existence', 'warn'],
    },
    {
      label: 'import validation',
      command: 'just',
      args: ['gate-imports', 'strict'],
    },
    {
      label: 'typecheck',
      command: 'just',
      args: ['typecheck'],
    },
    {
      label: 'unit test',
      command: 'just',
      args: ['test'],
    },
    {
      label: 'platform validation',
      command: 'just',
      args: ['validate-platform'],
    },
  ],
  ci: [
    {
      label: 'full verify',
      command: 'just',
      args: ['verify'],
    },
    {
      label: 'platform doctor',
      command: 'just',
      args: ['platform-doctor'],
    },
    {
      label: 'validate state',
      command: 'bun',
      args: ['run', 'scripts/validate-state.ts', '--mode', 'strict'],
    },
    {
      label: 'validate contracts',
      command: 'bun',
      args: ['run', 'scripts/validate-contracts.ts', '--mode', 'strict'],
    },
    {
      label: 'validate imports',
      command: 'bun',
      args: ['run', 'scripts/validate-imports.ts', '--mode', 'strict'],
    },
    {
      label: 'boundary check',
      command: 'bun',
      args: ['run', 'scripts/boundary-check.ts'],
    },
  ],
  release: [
    {
      label: 'ci gate',
      command: 'bun',
      args: ['run', 'scripts/gate.ts', 'ci', '--mode', 'strict'],
    },
    {
      label: 'release build',
      command: 'just',
      args: ['build-release'],
    },
  ],
};

function parseArgs(argv: string[]): { gate: GateName; mode: GateMode } {
  const gate = argv[0] as GateName | undefined;
  if (!gate || !(gate in gateDefinitions)) {
    printUsage();
    process.exit(1);
  }

  let mode: GateMode = gate === 'ci' || gate === 'release' ? 'strict' : 'warn';

  for (let index = 1; index < argv.length; index += 1) {
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
    printUsage();
    process.exit(1);
  }

  return { gate, mode };
}

function printUsage(): void {
  console.error('Usage: bun run scripts/gate.ts <local|prepush|ci|release> [--mode warn|strict]');
}

async function main(): Promise<number> {
  const { gate, mode } = parseArgs(process.argv.slice(2));
  const commands = gateDefinitions[gate];
  const failures: Array<{ label: string; exitCode: number; details: string }> = [];

  console.log(`=== gate-${gate} (${mode}) ===`);

  for (const step of commands) {
    console.log(`\n→ ${step.label}: ${step.command} ${step.args.join(' ')}`);
    const result = await run(step.command, step.args, { stdio: 'pipe' });

    if (result.output) {
      console.log(result.output);
    }

    if (result.success) {
      console.log(`✓ ${step.label}`);
      continue;
    }

    if (result.error) {
      console.error(result.error);
    }

    failures.push({
      label: step.label,
      exitCode: result.exitCode,
      details: result.error || result.output || 'no details',
    });

    console.error(`✗ ${step.label} failed with exit code ${result.exitCode}`);

    if (mode === 'strict') {
      console.error(`gate-${gate} blocked by ${step.label}`);
      return result.exitCode || 1;
    }
  }

  if (failures.length === 0) {
    console.log(`\n✓ gate-${gate} passed`);
    return 0;
  }

  console.warn(`\n! gate-${gate} completed with ${failures.length} warning(s)`);
  for (const failure of failures) {
    console.warn(`  - ${failure.label}: exit ${failure.exitCode}`);
  }

  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.stack || error.message : String(error);
    console.error(message);
    process.exit(1);
  });

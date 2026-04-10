/**
 * Comprehensive tests for generate-workflow.ts
 *
 * Covers: multi-provider harness dispatch, workflow type generation,
 * notification steps, CodeRun CRDs, dependency chains, and PR body harness table.
 */
import { test, expect, describe } from 'bun:test';
import { generateWorkflows } from './generate-workflow';
import type { GeneratedTask, TaskScaffold } from './types';

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const codingTask: GeneratedTask = {
  id: 2,
  title: 'Build Equipment Catalog Service',
  description: 'Rust/Axum service for equipment CRUD',
  dependencies: [1],
  agent: 'rex',
  stack: 'Rust 1.75+/Axum 0.7',
  difficulty_score: 7,
  subtasks: [
    { id: 1, title: 'Setup Axum router', description: '', dependencies: [], parallelizable: false },
    { id: 2, title: 'CRUD endpoints', description: '', dependencies: [1], parallelizable: true },
    { id: 3, title: 'Database migrations', description: '', dependencies: [], parallelizable: true },
  ],
};

const infraTask: GeneratedTask = {
  id: 1,
  title: 'Provision Core Infrastructure',
  description: 'Kubernetes + Helm bootstrap',
  dependencies: [],
  agent: 'bolt',
  stack: 'Kubernetes/Helm',
  difficulty_score: 5,
  subtasks: [
    { id: 1, title: 'Create namespace', description: '', dependencies: [], parallelizable: false },
    { id: 2, title: 'Deploy Helm chart', description: '', dependencies: [1], parallelizable: false },
  ],
};

const frontendTask: GeneratedTask = {
  id: 3,
  title: 'Build Website',
  description: 'Next.js frontend',
  dependencies: [],
  agent: 'blaze',
  stack: 'Next.js 14',
  difficulty_score: 6,
  subtasks: [],
};

const goTask: GeneratedTask = {
  id: 4,
  title: 'Build Rental System',
  description: 'Go gRPC service',
  dependencies: [1],
  agent: 'grizz',
  stack: 'Go 1.22+/gRPC',
  difficulty_score: 5,
  subtasks: [
    { id: 1, title: 'Proto definitions', description: '', dependencies: [], parallelizable: false },
  ],
};

const agentHarness = {
  rex:      { cli: 'Claude Code', provider: { name: 'Anthropic' }, models: [{ name: 'claude-opus-4-6-20260205' }, { name: 'claude-sonnet-4-6-20260514' }], baseUrl: 'https://api.anthropic.com', apiKey: 'ANTHROPIC_API_KEY' },
  bolt:     { cli: 'Codex',       provider: { name: 'OpenAI' },    models: [{ name: 'gpt-5.2-codex' }, { name: 'gpt-5.3-codex' }], baseUrl: 'https://api.openai.com/v1', apiKey: 'OPENAI_API_KEY' },
  blaze:    { cli: 'Cursor',      provider: { name: 'Cursor' },    models: [{ name: 'opus-4.6' }, { name: 'gpt-5.4' }], apiKey: 'CURSOR_API_KEY' },
  grizz:    { cli: 'Codex',       provider: { name: 'OpenAI' },    models: [{ name: 'gpt-5.2-codex' }, { name: 'gpt-5.3-codex' }], baseUrl: 'https://api.openai.com/v1', apiKey: 'OPENAI_API_KEY' },
  cipher:   { cli: 'Claude Code', provider: { name: 'Anthropic' }, models: [{ name: 'claude-opus-4-6-20260205' }, { name: 'claude-sonnet-4-6-20260514' }], baseUrl: 'https://api.anthropic.com', apiKey: 'ANTHROPIC_API_KEY' },
  cleo:     { cli: 'Claude Code', provider: { name: 'Anthropic' }, models: [{ name: 'claude-opus-4-6-20260205' }, { name: 'claude-sonnet-4-6-20260514' }], baseUrl: 'https://api.anthropic.com', apiKey: 'ANTHROPIC_API_KEY' },
  tess:     { cli: 'Codex',       provider: { name: 'OpenAI' },    models: [{ name: 'gpt-5.2-codex' }, { name: 'gpt-5.3-codex' }], baseUrl: 'https://api.openai.com/v1', apiKey: 'OPENAI_API_KEY' },
  _default: { cli: 'Claude Code', provider: { name: 'Anthropic' }, models: [{ name: 'claude-opus-4-6-20260205' }, { name: 'claude-sonnet-4-6-20260514' }], baseUrl: 'https://api.anthropic.com', apiKey: 'ANTHROPIC_API_KEY' },
};

const playConfig = {
  implementationMaxRetries: 10,
  qualityMaxRetries: 5,
  securityMaxRetries: 2,
  testingMaxRetries: 5,
  agentHarness,
};

const repoUrl = 'https://github.com/sigma-1/app';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function gen(tasks: GeneratedTask[], config = playConfig) {
  return generateWorkflows({
    expanded_tasks: tasks,
    scaffolds: [],
    config,
    repository_url: repoUrl,
  });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('generateWorkflows', () => {
  describe('workflow type generation', () => {
    test('coding task produces 4 workflow types', () => {
      const out = gen([codingTask]);
      expect(out.task_workflows).toHaveLength(1);
      const tw = out.task_workflows[0];
      expect(tw.task_type).toBe('task');
      expect(tw.workflow_yaml).toBeTruthy();
      expect(tw.quality_yaml).toBeTruthy();
      expect(tw.security_yaml).toBeTruthy();
      expect(tw.testing_yaml).toBeTruthy();
    });

    test('infra task produces 2 workflow types (no quality/testing)', () => {
      const out = gen([infraTask]);
      expect(out.task_workflows).toHaveLength(1);
      const tw = out.task_workflows[0];
      expect(tw.task_type).toBe('infra');
      expect(tw.workflow_yaml).toBeTruthy();
      expect(tw.security_yaml).toBeTruthy();
      expect(tw.quality_yaml).toBeUndefined();
      expect(tw.testing_yaml).toBeUndefined();
    });

    test('play_yaml is always generated', () => {
      const out = gen([infraTask]);
      expect(out.play_yaml).toBeTruthy();
      expect(out.play_yaml).toContain('name: play');
    });
  });

  describe('multi-provider harness dispatch', () => {
    test('rex tasks get claude harness', () => {
      const out = gen([codingTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Claude Code');
      expect(yaml).toContain('model: claude-opus-4-6-20260205');
      expect(yaml).toContain('default: "Claude Code"');
    });

    test('bolt tasks get codex harness', () => {
      const out = gen([infraTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Codex');
      expect(yaml).toContain('model: gpt-5.2-codex');
      expect(yaml).toContain('default: "Codex"');
    });

    test('blaze tasks get cursor harness', () => {
      const out = gen([frontendTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Cursor');
      expect(yaml).toContain('model: opus-4.6');
      expect(yaml).toContain('default: "Cursor"');
    });

    test('grizz tasks get codex harness', () => {
      const out = gen([goTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Codex');
      expect(yaml).toContain('model: gpt-5.2-codex');
    });

    test('fallback inputs are included', () => {
      const out = gen([codingTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('fallback_cli');
      expect(yaml).toContain('default: "codex"');
      expect(yaml).toContain('fallback_model');
      expect(yaml).toContain('default: "gpt-5.2-codex"');
    });

    test('quality workflow uses cleo harness (claude)', () => {
      const out = gen([codingTask]);
      const qYaml = out.task_workflows[0].quality_yaml!;
      expect(qYaml).toContain('agent: cleo');
      expect(qYaml).toContain('cli: Claude Code');
      expect(qYaml).toContain('model: claude-opus-4-6-20260205');
    });

    test('security workflow uses cipher harness (claude)', () => {
      const out = gen([codingTask]);
      const sYaml = out.task_workflows[0].security_yaml;
      expect(sYaml).toContain('agent: cipher');
      expect(sYaml).toContain('cli: Claude Code');
      expect(sYaml).toContain('model: claude-opus-4-6-20260205');
    });

    test('testing workflow uses tess harness (codex)', () => {
      const out = gen([codingTask]);
      const tYaml = out.task_workflows[0].testing_yaml!;
      expect(tYaml).toContain('agent: tess');
      expect(tYaml).toContain('cli: Codex');
      expect(tYaml).toContain('model: gpt-5.2-codex');
    });

    test('unknown agent falls back to _default harness', () => {
      const unknownTask: GeneratedTask = {
        ...codingTask,
        id: 99,
        agent: 'unknown-agent',
        stack: 'python',
      };
      const out = gen([unknownTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('default: "Claude Code"');
      expect(yaml).toContain('default: "claude-opus-4-6-20260205"');
    });

    test('no agentHarness config falls back to DEFAULT_HARNESS', () => {
      const out = gen([codingTask], {
        implementationMaxRetries: 10,
        qualityMaxRetries: 5,
      });
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('default: "Claude Code"');
      expect(yaml).toContain('default: "claude-opus-4-6-20260205"');
    });
  });

  describe('play workflow CodeRun CRDs', () => {
    test('CodeRun CRD uses per-task harness via shell variables', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;

      // bolt → codex: PRIMARY_CLI should be set to codex
      expect(play).toContain('PRIMARY_CLI="Codex"');
      expect(play).toContain('PRIMARY_MODEL="gpt-5.2-codex"');

      // rex → claude: PRIMARY_CLI should be set to claude
      expect(play).toContain('PRIMARY_CLI="Claude Code"');
      expect(play).toContain('PRIMARY_MODEL="claude-opus-4-6-20260205"');
    });

    test('CodeRun CRD does not use {{inputs.cli}} for cliType', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;
      expect(play).not.toContain('cliType: {{inputs.cli}}');
    });

    test('CodeRun labels include cli variable reference', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      expect(play).toContain('cto.5dlabs.ai/cli:');
    });

    test('CodeRun includes subtasks', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('subtasks:');
      expect(play).toContain('Setup Axum router');
      expect(play).toContain('CRUD endpoints');
    });
  });

  describe('play workflow notification steps', () => {
    test('play-start notification is first step', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;
      const stepsSection = play.split('steps:')[1];
      const firstStep = stepsSection.match(/- id: (\S+)/);
      expect(firstStep?.[1]).toBe('notify-play-start');
    });

    test('play-start includes task count in Discord message', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('🎬 Play started — 2 tasks dispatching');
    });

    test('play-start includes harness table in Linear activity', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('2 tasks dispatching to 2 harnesses');
      expect(play).toContain('| Task | Title | Agent | CLI | Model | Type |');
    });

    test('each task gets a notify-task-N-start step', () => {
      const out = gen([infraTask, codingTask, frontendTask]);
      const play = out.play_yaml;
      expect(play).toContain('id: notify-task-1-start');
      expect(play).toContain('id: notify-task-2-start');
      expect(play).toContain('id: notify-task-3-start');
    });

    test('task dispatch notification includes harness details', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      expect(play).toContain('→ Codex (gpt-5.2-codex) — agent: bolt');
    });

    test('run-task depends on notify step, not directly on gate', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      // run-task-1 should depend on notify-task-1-start
      const runMatch = play.match(/id: run-task-1[\s\S]*?depends_on: \[([^\]]+)\]/);
      expect(runMatch?.[1]).toContain('notify-task-1-start');
    });

    test('gate step includes Discord + Linear notifications', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('✅ Task 2');
      expect(play).toContain('gate passed');
      expect(play).toContain('bridge-notify');
      expect(play).toContain('linear-activity');
    });

    test('play-complete step includes notifications', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      expect(play).toContain('🏁 Play complete');
      expect(play).toContain('bridge-notify --from morgan');
    });

    test('all notifications use || true (non-blocking)', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      const bridgeLines = play.split('\n').filter((l: string) => l.includes('bridge-notify'));
      for (const line of bridgeLines) {
        expect(line).toContain('|| true');
      }
    });

    test('discord_channel input defaults to play', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      expect(play).toContain('discord_channel:');
      expect(play).toContain('default: "play"');
    });
  });

  describe('play workflow sub-workflow dispatch', () => {
    test('security sub-workflow uses cipher harness', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      // security step should pass cipher's CLI (Claude Code)
      const secLines = play.split('\n').filter((l: string) =>
        l.includes("security.lobster.yaml") || (l.includes("--arg cli") && l.includes("'Claude Code'"))
      );
      expect(secLines.length).toBeGreaterThan(0);
    });

    test('quality sub-workflow uses cleo harness', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      const qualSection = play.split('id: quality-task-2')[1]?.split('id:')[0] ?? '';
      expect(qualSection).toContain("--arg cli 'Claude Code'");
    });

    test('testing sub-workflow uses tess harness (codex)', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      const testSection = play.split('id: testing-task-2')[1]?.split('id:')[0] ?? '';
      expect(testSection).toContain("--arg cli 'Codex'");
    });

    test('infra task has security but no quality/testing in play', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      expect(play).toContain('id: security-task-1');
      expect(play).not.toContain('id: quality-task-1');
      expect(play).not.toContain('id: testing-task-1');
    });

    test('coding task has all three checks in play', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('id: security-task-2');
      expect(play).toContain('id: quality-task-2');
      expect(play).toContain('id: testing-task-2');
    });
  });

  describe('dependency chains', () => {
    test('task with dependencies gates on predecessor', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;
      // Task 2 depends on task 1 → notify-task-2-start depends on gate-task-1
      const notifySection = play.split('id: notify-task-2-start')[1]?.split('name:')[0] ?? '';
      expect(notifySection).toContain('gate-task-1');
    });

    test('task without dependencies has no depends_on on notify step', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      const notifySection = play.split('id: notify-task-1-start')[1]?.split('command:')[0] ?? '';
      expect(notifySection).not.toContain('depends_on');
    });

    test('gate step depends on all check steps', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      const gateSection = play.split('id: gate-task-2')[1]?.split('name:')[0] ?? '';
      expect(gateSection).toContain('security-task-2');
      expect(gateSection).toContain('quality-task-2');
      expect(gateSection).toContain('testing-task-2');
    });

    test('play-complete depends on all gates', () => {
      const out = gen([infraTask, codingTask, frontendTask]);
      const play = out.play_yaml;
      const completeSection = play.split('id: play-complete')[1] ?? '';
      expect(completeSection).toContain('gate-task-1');
      expect(completeSection).toContain('gate-task-2');
      expect(completeSection).toContain('gate-task-3');
    });
  });

  describe('metadata in generated workflows', () => {
    test('implementation workflow metadata includes cli and model', () => {
      const out = gen([codingTask]);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Claude Code');
      expect(yaml).toContain('model: claude-opus-4-6-20260205');
    });

    test('quality workflow metadata includes cli and model', () => {
      const out = gen([codingTask]);
      const yaml = out.task_workflows[0].quality_yaml!;
      expect(yaml).toContain('cli: Claude Code');
      expect(yaml).toContain('model: claude-opus-4-6-20260205');
    });

    test('security workflow metadata includes cli and model', () => {
      const out = gen([codingTask]);
      const yaml = out.task_workflows[0].security_yaml;
      expect(yaml).toContain('cli: Claude Code');
      expect(yaml).toContain('model: claude-opus-4-6-20260205');
    });

    test('testing workflow metadata includes cli and model', () => {
      const out = gen([codingTask]);
      const yaml = out.task_workflows[0].testing_yaml!;
      expect(yaml).toContain('cli: Codex');
      expect(yaml).toContain('model: gpt-5.2-codex');
    });
  });

  describe('harness summary', () => {
    test('play-complete echoes harness summary', () => {
      const out = gen([infraTask, codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('task-1(bolt):Codex');
      expect(play).toContain('task-2(rex):Claude Code');
    });

    test('gate output JSON includes cli and model', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('cli: "Claude Code"');
      expect(play).toContain('model: "claude-opus-4-6-20260205"');
    });
  });

  describe('multi-task mixed harness play', () => {
    test('10-task play dispatches 3 different harnesses', () => {
      const tasks: GeneratedTask[] = [
        { ...infraTask, id: 1, agent: 'bolt' },
        { ...codingTask, id: 2, agent: 'rex', dependencies: [1] },
        { ...goTask, id: 3, agent: 'grizz', dependencies: [1] },
        { ...frontendTask, id: 4, agent: 'blaze', dependencies: [] },
      ];
      const out = gen(tasks);

      expect(out.task_workflows).toHaveLength(4);
      // Check PRIMARY_CLI assignments for each harness type
      expect(out.play_yaml).toContain('PRIMARY_CLI="Codex"');   // bolt, grizz
      expect(out.play_yaml).toContain('PRIMARY_CLI="Claude Code"');  // rex
      expect(out.play_yaml).toContain('PRIMARY_CLI="Cursor"');  // blaze

      // Harness table in play-start should mention 3 harnesses
      expect(out.play_yaml).toContain('3 harnesses');
    });
  });

  describe('fallback cascade', () => {
    test('run-task contains apply_coderun and wait_coderun functions', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('apply_coderun()');
      expect(play).toContain('wait_coderun()');
    });

    test('primary and fallback CLI/model are defined', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      // rex → primary claude, fallback codex
      expect(play).toContain('PRIMARY_CLI="Claude Code"');
      expect(play).toContain('PRIMARY_MODEL="claude-opus-4-6-20260205"');
      expect(play).toContain('FALLBACK_CLI="codex"');
      expect(play).toContain('FALLBACK_MODEL="gpt-5.2-codex"');
    });

    test('fallback attempt triggers on primary failure', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('Primary $PRIMARY_CLI failed');
      expect(play).toContain('falling back to');
      expect(play).toContain('fallback-trigger');
    });

    test('fallback CodeRun has -fallback suffix', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain(`play-task-2-rex-fallback`);
    });

    test('fallback deletes failed primary CodeRun', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('kubectl delete "coderun/$RUN_NAME"');
      expect(play).toContain('--ignore-not-found=true');
    });

    test('fallback sends Discord notification', () => {
      const out = gen([infraTask]);
      const play = out.play_yaml;
      expect(play).toContain('primary Codex failed, falling back to codex');
    });

    test('output JSON uses USED_CLI variable for actual harness used', () => {
      const out = gen([codingTask]);
      const play = out.play_yaml;
      expect(play).toContain('--arg cli "$USED_CLI"');
      expect(play).toContain('--arg model "$USED_MODEL"');
    });
  });

  describe('slim harness resolution (model-providers.json refs)', () => {
    const catalog = {
      providers: {
        Anthropic: { enabled: true, baseUrl: 'https://api.anthropic.com', apiKey: 'ANTHROPIC_API_KEY', credits: 250000 },
        OpenAI:    { enabled: true, baseUrl: 'https://api.openai.com/v1', apiKey: 'OPENAI_API_KEY', credits: 180000 },
      },
      models: {
        'claude-opus-4-6-20260205':   { enabled: true, provider: 'Anthropic', thinkingLevel: 'high', score: 96 },
        'claude-sonnet-4-6-20260514': { enabled: true, provider: 'Anthropic', thinkingLevel: 'medium', score: 91 },
        'gpt-5.2-codex':              { enabled: true, provider: 'OpenAI', thinkingLevel: 'medium', score: 92 },
        'gpt-5.3-codex':              { enabled: true, provider: 'OpenAI', thinkingLevel: 'high', score: 94 },
        'disabled-model':             { enabled: false, provider: 'OpenAI', thinkingLevel: 'low', score: 50 },
      },
      clis: {
        'Claude Code': { enabled: true, provider: 'Anthropic', models: ['claude-opus-4-6-20260205', 'claude-sonnet-4-6-20260514'] },
        'Codex':       { enabled: true, provider: 'OpenAI',    models: ['gpt-5.2-codex', 'gpt-5.3-codex', 'disabled-model'] },
      },
    };

    // Slim harness: just { cli, model } — resolver should expand to full shape
    const slimHarness: Record<string, any> = {
      rex:      { cli: 'Claude Code', model: 'claude-opus-4-6-20260205' },
      bolt:     { cli: 'Codex',       model: 'gpt-5.2-codex' },
      _default: { cli: 'Claude Code', model: 'claude-opus-4-6-20260205' },
    };

    const slimConfig = {
      implementationMaxRetries: 10,
      qualityMaxRetries: 5,
      agentHarness: slimHarness,
      modelProviders: catalog,
    };

    test('slim harness resolves CLI and model', () => {
      const out = gen([codingTask], slimConfig);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Claude Code');
      expect(yaml).toContain('model: claude-opus-4-6-20260205');
    });

    test('slim harness resolves provider baseUrl and apiKey', () => {
      const out = gen([codingTask], slimConfig);
      const play = out.play_yaml;
      expect(play).toContain('PRIMARY_CLI="Claude Code"');
      expect(play).toContain('PRIMARY_MODEL="claude-opus-4-6-20260205"');
    });

    test('disabled models are filtered from resolved models list', () => {
      const out = gen([infraTask], slimConfig);
      const play = out.play_yaml;
      // bolt uses Codex — disabled-model should NOT appear
      expect(play).toContain('PRIMARY_CLI="Codex"');
      expect(play).not.toContain('disabled-model');
    });

    test('slim harness without catalog still works', () => {
      const noCatalogConfig = {
        implementationMaxRetries: 10,
        qualityMaxRetries: 5,
        agentHarness: slimHarness,
      };
      const out = gen([codingTask], noCatalogConfig);
      const yaml = out.task_workflows[0].workflow_yaml;
      expect(yaml).toContain('cli: Claude Code');
      expect(yaml).toContain('model: claude-opus-4-6-20260205');
    });
  });
});

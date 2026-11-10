// 连通性验证：确认嵌入后端可达、维度符合预期、一条记忆能建成并召回
//
// 运行：node test/bin/probe.cjs [--profile http]
//
// 产物：test/out/<时间戳>-probe-<profile>/（库与日志随产物走，用完可删）

const path = require('path');
const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const { listConfigs } = require('../lib/config.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};

const PROFILE = arg('profile', 'http');

(async () => {
  if (!listConfigs().includes(PROFILE)) {
    throw new Error(`未知配置 ${PROFILE}，可用：${listConfigs().join(', ')}`);
  }

  const startedAt = new Date().toISOString();
  const run = paths.newRun('probe', PROFILE);

  const t0 = Date.now();
  const { api, dim } = await harness.init({ profile: PROFILE, run });
  console.log(`init 成功：${Date.now() - t0}ms（配置 ${PROFILE}，维度 ${dim}）`);

  const t1 = Date.now();
  const ballId = await api.createBall('用户的生日是 3 月 5 日', null, 3.0);
  console.log(`建球（含一次远程嵌入）：${Date.now() - t1}ms，id = ${ballId}`);

  const t2 = Date.now();
  const r = await api.recall('用户的生日是哪天', 1.0);
  console.log(`回忆：${Date.now() - t2}ms，注入块 ${r.blocks.length} 个，token ${r.stats.tokenCount}`);
  console.log(
    `  点亮 ${r.stats.litCount}，起始 ${r.stats.startCount}，` +
      `被挡 ${r.stats.blockedPipeCount}，强化 ${r.stats.strengthenedCount}`,
  );
  for (const b of r.blocks) {
    console.log(`  - [${b.confidence}] ${b.text.slice(0, 60)}`);
  }

  api.shutdown();

  harness.writeMeta(run, {
    script: 'probe',
    tag: PROFILE,
    profile: PROFILE,
    startedAt,
    elapsedMs: Date.now() - t0,
    dim,
  });
  console.log(`\n产物目录：${paths.rel(run.dir)}`);
})().catch((e) => {
  console.error('FAILED', e.message);
  process.exit(1);
});

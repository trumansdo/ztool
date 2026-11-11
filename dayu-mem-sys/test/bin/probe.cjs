// 连通性冒烟：napi 能加载、配置能解析、库能开、一次完整链路能跑通，且有断言
//
// 这是**部署守门员**：它验的不是召回质量，是「装没装对」。失败即红，可进 CI。
// 与 bin/smoke.cjs 的分工：smoke 在长期库上验统计不变量（球数单调、无孤儿管道），
// probe 在干净库里验「这条链路活着」——两者都需要断言，所以都在 bin/。
// 想要规模/参数的对照数字，去 lab/（那里没有 pass/fail）。
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

/// 单次 recall 的耗时上限。放宽到 30s 是有意的：http profile 会打远程嵌入，
/// 网络抖动不该让部署守门员变红；但真挂住（比如 dll 版本不符死循环）必须暴露。
const RECALL_TIMEOUT_MS = 30_000;

// ---------- 不变量 ----------

function checkInvariants({ dim, ballId, recall, elapsedMs }) {
  const checks = [];
  const push = (name, ok, detail) => checks.push({ name, ok, detail });

  push('嵌入维度非零', Number.isInteger(dim) && dim > 0, `dim = ${dim}`);

  push(
    '建球返回有效 id',
    Number.isInteger(ballId) && ballId > 0,
    `id = ${ballId}`,
  );

  push(
    '至少选中一个起始球',
    recall.stats.startCount >= 1,
    `起始 ${recall.stats.startCount}`,
  );

  push(
    '至少点亮一个球',
    recall.stats.litCount >= 1,
    `点亮 ${recall.stats.litCount} / 被挡 ${recall.stats.blockedPipeCount}`,
  );

  push(
    '至少注入一个块',
    recall.blocks.length >= 1,
    `注入 ${recall.blocks.length} 块 / ${recall.stats.tokenCount} token`,
  );

  const emptyText = recall.blocks.filter((b) => !b.text || !b.text.trim());
  push('注入块内容非空', emptyText.length === 0, `${recall.blocks.length} 块全部非空`);

  const badConfidence = recall.blocks.filter((b) => !b.confidence);
  push('每个块都带置信档位', badConfidence.length === 0, `档位 ${[...new Set(recall.blocks.map((b) => b.confidence))].join('/') || '无'}`);

  push(
    '召回延迟在阈值内',
    elapsedMs < RECALL_TIMEOUT_MS,
    `${elapsedMs}ms < ${RECALL_TIMEOUT_MS}ms`,
  );

  return checks;
}

// ---------- 主流程 ----------

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
  const recallMs = Date.now() - t2;
  console.log(`回忆：${recallMs}ms，注入块 ${r.blocks.length} 个，token ${r.stats.tokenCount}`);
  console.log(
    `  点亮 ${r.stats.litCount}，起始 ${r.stats.startCount}，` +
      `被挡 ${r.stats.blockedPipeCount}，强化 ${r.stats.strengthenedCount}`,
  );
  for (const b of r.blocks) {
    console.log(`  - [${b.confidence}] ${b.text.slice(0, 60)}`);
  }

  const checks = checkInvariants({ dim, ballId, recall: r, elapsedMs: recallMs });

  api.shutdown();

  harness.writeMeta(run, {
    script: 'probe',
    tag: PROFILE,
    profile: PROFILE,
    startedAt,
    elapsedMs: Date.now() - t0,
    dim,
    checks,
    checks_ok: checks.every((c) => c.ok),
  });

  console.log('\n=== 不变量 ===');
  for (const c of checks) {
    console.log(`  ${c.ok ? '✅' : '❌'} ${c.name.padEnd(12)} ${c.detail}`);
  }

  console.log(`\n产物目录：${paths.rel(run.dir)}`);

  if (!checks.every((c) => c.ok)) process.exit(1);
})().catch((e) => {
  console.error('FAILED', e.message);
  process.exit(1);
});

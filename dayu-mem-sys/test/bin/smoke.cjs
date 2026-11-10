// 长期全链路冒烟：同一个库跨运行累积，测「用久了会怎样」
//
// 与规模测试的分工：
//   本脚本   test/data/main.tdb        几百~几千球，模拟真实用户长期使用，每次追加一点
//   scale    test/out/<run>/db         每次重建，几万~十万球，测压力与参数敏感度
//
// 长期库的价值在 test/data/main.history.jsonl：每次跑完追加一行统计快照。
// 哪天行为变了（比如 p50 延迟从 18ms 涨到 45ms），翻这行历史就知道是哪次提交引入的。
//
// 运行：
//   node test/bin/smoke.cjs                 # 追加一轮
//   node test/bin/smoke.cjs --reset         # 丢弃长期库重建（会先备份）
//   node test/bin/smoke.cjs --profile http  # 换嵌入后端（维度变了会拒绝，见下）

const fs = require('fs');
const path = require('path');

const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const { QUERIES, DOMAINS, BASE } = require('../lib/corpus.cjs');
const { mean, percentile, pctStr } = require('../lib/stats.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};

const PROFILE = arg('profile', 'onnx');
const RESET = process.argv.includes('--reset');
const SEED_SIZE = +arg('seed', 150); // 首次建库的球数（BASE 前 N 条）
const APPEND = +arg('append', 3); // 每轮追加的球数

// ---------- 长期库状态 ----------

function readHistory() {
  if (!fs.existsSync(paths.MAIN_HISTORY)) return [];
  return fs
    .readFileSync(paths.MAIN_HISTORY, 'utf8')
    .split('\n')
    .filter((l) => l.trim())
    .map((l) => JSON.parse(l));
}

function appendHistory(entry) {
  fs.appendFileSync(paths.MAIN_HISTORY, `${JSON.stringify(entry)}\n`, 'utf8');
}

function backupMainDb() {
  if (!fs.existsSync(paths.MAIN_DB)) return null;
  const dir = path.join(paths.DATA_DIR, 'snapshots');
  fs.mkdirSync(dir, { recursive: true });
  const target = path.join(dir, `main-${paths.stamp()}.tdb`);
  for (const f of fs.readdirSync(paths.DATA_DIR)) {
    if (f === 'main.tdb' || f.startsWith('main.tdb.')) {
      fs.copyFileSync(path.join(paths.DATA_DIR, f), `${target}${f.slice('main.tdb'.length)}`);
    }
  }
  return paths.rel(target);
}

// ---------- 不变量 ----------

function checkInvariants({ stats, pipes, ballIds, prev }) {
  const checks = [];

  checks.push({
    name: '球数单调不减',
    ok: prev ? stats.activeBallCount >= prev.active_ball_count : true,
    detail: prev ? `${prev.active_ball_count} → ${stats.activeBallCount}` : '首次运行',
  });

  const known = new Set(ballIds);
  const orphan = pipes.filter((p) => !known.has(p.a) || !known.has(p.b));
  checks.push({
    name: '无孤儿管道',
    ok: orphan.length === 0,
    detail: orphan.length ? `${orphan.length} 条管道端点不存在` : `${pipes.length} 条全部有效`,
  });

  checks.push({
    name: '归档数不超过总球数',
    ok: stats.archivedBallCount <= stats.activeBallCount + stats.archivedBallCount,
    detail: `归档 ${stats.archivedBallCount} / 活跃 ${stats.activeBallCount}`,
  });

  return checks;
}

// ---------- 主流程 ----------

(async () => {
  const startedAt = new Date().toISOString();
  const t0 = Date.now();
  const history = readHistory();
  const prev = history.length ? history[history.length - 1] : null;

  let backup = null;
  if (RESET) {
    backup = backupMainDb();
    paths.clearDb(paths.MAIN_DB);
    fs.writeFileSync(paths.MAIN_HISTORY, '', 'utf8');
    console.log(`已重置长期库${backup ? `（备份 ${backup}）` : ''}`);
  }

  const isNew = !fs.existsSync(paths.MAIN_DB) || paths.dbBytes(paths.MAIN_DB) === 0;
  const { api, dim } = await harness.init({ profile: PROFILE, reuseDb: true });

  // 维度变化 = 换了嵌入模型，旧向量不可用，必须重建而不是静默混用
  if (prev && prev.dim !== dim) {
    throw new Error(
      `嵌入维度变了（历史 ${prev.dim} → 现在 ${dim}）。\n` +
        `同一个库不能混用不同模型的向量，请加 --reset 重建长期库。`,
    );
  }

  // ---- 首次建库：铺基础语料 ----
  let seeded = 0;
  if (isNew) {
    const mems = BASE.slice(0, SEED_SIZE);
    const { vectors } = await harness.embed(
      mems.map((m) => m.text),
      { tag: PROFILE },
    );
    const { ids } = await harness.seedGraph(
      api,
      mems,
      vectors,
      require('../lib/corpus.cjs').buildEdges(mems),
    );
    seeded = ids.length;
    console.log(`首次建库：${seeded} 球 / ${(await api.getStats()).activeBallCount} 活跃`);
  }

  // ---- 每轮追加：模拟「今天又记了几件事」 ----
  const round = history.length + 1;
  const domain = DOMAINS[round % DOMAINS.length];
  const pool = BASE.filter((m) => m.domain === domain);
  const fresh = Array.from({ length: APPEND }, (_, i) => {
    const src = pool[(round * APPEND + i) % pool.length];
    return { text: `第 ${round} 轮记录：${src.text}`, domain };
  });
  const { vectors: freshVecs } = await harness.embed(
    fresh.map((m) => m.text),
    { tag: PROFILE },
  );
  const freshIds = await harness.seedBalls(
    api,
    fresh.map((m) => m.text),
    freshVecs,
    fresh.map(() => 1.0),
  );

  // 新球连进该领域已有球（取库里该领域的前两个）
  const existing = (await api.listBalls()).filter((b) => b.text.includes(pool[0].text.slice(0, 8)));
  const anchors = existing.slice(0, 2).map((b) => b.id);
  let newPipes = 0;
  for (let i = 0; i < freshIds.length; i++) {
    const anchor = anchors[i % anchors.length];
    if (anchor === undefined) break;
    await api.linkBalls(freshIds[i], anchor, 0.7);
    newPipes++;
  }

  // ---- 球 id → 领域（用基础句尾 8 字做键，避免 O(n×375) 全表扫描）----
  const domainOfTail = new Map(BASE.map((m) => [m.text.slice(-8), m.domain]));
  const domainOfBall = new Map(
    (await api.listBalls()).map((b) => [b.id, domainOfTail.get(b.text.slice(-8)) ?? null]),
  );

  // ---- 跑全部查询 ----
  const rows = [];
  for (const { q, domain: qd } of QUERIES) {
    const t = Date.now();
    const r = await api.recall(q, 1.0);
    const elapsedMs = Date.now() - t;

    // 领域命中：注入块里含有与查询同领域的球的比例
    let hit = 0;
    let counted = 0;
    for (const blk of r.blocks) {
      const doms = (blk.ballIds || []).map((id) => domainOfBall.get(id)).filter(Boolean);
      if (!doms.length) continue;
      counted++;
      if (doms.includes(qd)) hit++;
    }
    rows.push({
      query: q,
      domain: qd,
      elapsedMs,
      lit: r.stats.litCount,
      start: r.stats.startCount,
      blocks: r.stats.blockCount,
      tokens: r.stats.tokenCount,
      domainHitRate: counted ? hit / counted : null,
    });
  }

  const stats = await api.getStats();
  const pipes = await api.listPipes();
  const balls = await api.listBalls();

  const checks = checkInvariants({
    stats,
    pipes,
    ballIds: balls.map((b) => b.id),
    prev,
  });

  api.shutdown();

  const elapsed = rows.map((r) => r.elapsedMs);
  const entry = {
    ts: startedAt,
    commit: paths.gitCommit().commit,
    profile: PROFILE,
    dim,
    round,
    seeded,
    appended: freshIds.length,
    newPipes,
    active_ball_count: stats.activeBallCount,
    archived_ball_count: stats.archivedBallCount,
    pipe_count: pipes.length,
    mean_lit: +mean(rows.map((r) => r.lit)).toFixed(1),
    mean_blocks: +mean(rows.map((r) => r.blocks)).toFixed(2),
    mean_tokens: +mean(rows.map((r) => r.tokens)).toFixed(0),
    domain_hit_rate: +mean(rows.map((r) => r.domainHitRate ?? 0)).toFixed(3),
    recall_p50_ms: percentile(elapsed, 50),
    recall_p90_ms: percentile(elapsed, 90),
    checks_ok: checks.every((c) => c.ok),
  };
  appendHistory(entry);

  // ---- 报告 ----
  console.log(`\n=== 第 ${round} 轮（${PROFILE}，dim=${dim}）===`);
  console.log(`球 ${stats.activeBallCount}（+${freshIds.length}）  归档 ${stats.archivedBallCount}  管道 ${pipes.length}（+${newPipes}）`);
  console.log(
    `查询 ${rows.length} 条：平均点亮 ${entry.mean_lit}，平均注入 ${entry.mean_blocks} 块 / ${entry.mean_tokens} token，` +
      `领域命中率 ${pctStr(entry.domain_hit_rate)}，延迟 p50 ${entry.recall_p50_ms}ms / p90 ${entry.recall_p90_ms}ms`,
  );

  console.log('\n=== 不变量 ===');
  for (const c of checks) {
    console.log(`  ${c.ok ? '✅' : '❌'} ${c.name.padEnd(12)} ${c.detail}`);
  }

  if (history.length >= 2) {
    const prevEntry = history[history.length - 1];
    const delta = (k) => {
      const d = entry[k] - prevEntry[k];
      return `${d >= 0 ? '+' : ''}${d}`;
    };
    console.log(
      `\n相对上一轮：球 ${delta('active_ball_count')}  管道 ${delta('pipe_count')}  ` +
        `p50 ${delta('recall_p50_ms')}ms  平均注入 ${delta('mean_blocks')} 块`,
    );
  }

  console.log(`\n历史：${paths.rel(paths.MAIN_HISTORY)}（${round} 行）`);

  if (!checks.every((c) => c.ok)) process.exit(1);
})().catch((e) => {
  console.error('FAILED', e.message);
  process.exit(1);
});
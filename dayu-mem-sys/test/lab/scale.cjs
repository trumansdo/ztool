// 规模测量：几十 / 几百 / 几千 / 几万 / 十万球
//
// 这是**测量**不是测试：产出是一张规模×参数的表格（延迟、命中率、图规模），
// 没有 pass/fail，跑完永远退出码 0。所以它住 lab/ 而不是 bin/。
// 要「规模上不出错」的判定，用 bin/smoke.cjs 的不变量。
//
// 语料：test/corpus.cjs（15 领域 × 25 条 + 30 条查询），展开规则见 lib/corpus.cjs
// 每次运行一个干净的库，落在 test/out/<时间戳>-scale-<profile>/db/
//
// 运行：
//   node test/lab/scale.cjs --profile http --balls 100
//   node test/lab/scale.cjs --profile http --ladder
//   node test/lab/scale.cjs --profile http --ladder --skip 100000
//   node test/lab/scale.cjs --profile http --balls 5000 --budgets 1.0,2.5,10
//   node test/lab/scale.cjs --profile http --balls 5000 --tops 3,5,10
//
// 产物：test/out/<run>/{meta.json, result.json, db/, logs/}

const fs = require('fs');

const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const datasetLib = require('../lib/dataset.cjs');
const { mean } = require('../lib/stats.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};
const listArg = (name) =>
  arg(name, '')
    ? arg(name, '')
        .split(',')
        .map((s) => +s.trim())
    : null;

const PROFILE = arg('profile', 'http');
const DATASET = arg('dataset', 'main');
// 数据集在模块级加载：runOne() 每组建一次库，但数据集只有一份
const ds = datasetLib.load(DATASET);
const LADDER = process.argv.includes('--ladder');
const SKIP = +arg('skip', 0);
const BUDGETS = listArg('budgets');
const TOPS = listArg('tops');
const EMBED_CHUNK = +arg('chunk', 64);

// ---------- 单组 ----------

async function runOne(cfg) {
  const startedAt = new Date().toISOString();
  const run = paths.newRun('scale', `${PROFILE}-${cfg.balls}b-b${cfg.budget}-t${cfg.startTopN}`);

  const tInit = Date.now();
  const { api, dim, identity } = await harness.init({
    profile: PROFILE,
    run,
    overrides: {
      total_water_budget: cfg.budget,
      start_top_n: cfg.startTopN,
    },
  });
  const initMs = Date.now() - tInit;

  // 1. 数据集 + 嵌入（带缓存）
  const mems = datasetLib.memories(ds, cfg.balls);
  const tEmbed = Date.now();
  const emb = await harness.embed(
    mems.map((m) => m.text),
    { identity, dataset: ds, chunk: EMBED_CHUNK },
  );
  const embedMs = Date.now() - tEmbed;

  // 2. 建图
  const tCreate = Date.now();
  const { ids, pipes } = await harness.seedGraph(api, mems, emb.vectors, datasetLib.edges(ds, cfg.balls));
  const createMs = Date.now() - tCreate;

  // 3. 查询
  const domainOf = new Map(mems.map((m, i) => [ids[i], m.domain]));
  const rows = [];
  for (const { q, domain } of ds.queries) {
    const t = Date.now();
    let r;
    try {
      r = await api.recall(q, 1.0);
    } catch (e) {
      rows.push({ query: q, domain, error: e.message, elapsedMs: Date.now() - t });
      continue;
    }
    const elapsedMs = Date.now() - t;
    let hit = 0;
    let counted = 0;
    for (const blk of r.blocks) {
      const doms = (blk.ballIds || []).map((id) => domainOf.get(id)).filter(Boolean);
      if (!doms.length) continue;
      counted++;
      if (doms.includes(domain)) hit++;
    }
    rows.push({
      query: q,
      domain,
      elapsedMs,
      start: r.stats.startCount,
      lit: r.stats.litCount,
      blocked: r.stats.blockedPipeCount,
      strengthened: r.stats.strengthenedCount,
      blocks: r.stats.blockCount,
      tokens: r.stats.tokenCount,
      hitBlocks: hit,
      countedBlocks: counted,
    });
  }

  const stats = await api.getStats();
  api.shutdown();

  const ok = rows.filter((x) => !x.error);
  const avg = (k) => (ok.length ? mean(ok.map((x) => x[k])) : 0);
  const hitTotal = ok.reduce((s, x) => s + x.hitBlocks, 0);
  const cntTotal = ok.reduce((s, x) => s + x.countedBlocks, 0);

  const result = {
    dataset: ds.id,
    datasetSourceHash: ds.sourceHash.slice(0, 8),
    balls: cfg.balls,
    pipes,
    dim,
    budget: cfg.budget,
    sig: cfg.sig,
    startTopN: cfg.startTopN,
    initMs,
    embedMs,
    embedUnique: emb.unique,
    embedMissing: emb.missing,
    embedApiCalls: emb.apiCalls,
    createMs,
    dbSizeMB: +(paths.dbBytes(run.dbPath) / 1048576).toFixed(1),
    avgStart: +avg('start').toFixed(1),
    avgLit: +avg('lit').toFixed(2),
    avgBlocked: +avg('blocked').toFixed(1),
    avgStrengthened: +avg('strengthened').toFixed(1),
    avgBlocks: +avg('blocks').toFixed(2),
    avgTokens: +avg('tokens').toFixed(1),
    avgRecallMs: +avg('elapsedMs').toFixed(0),
    domainHitRate: cntTotal ? +(hitTotal / cntTotal).toFixed(3) : null,
    failedQueries: rows.length - ok.length,
    stats,
    rows,
  };

  harness.writeMeta(run, {
    script: 'scale',
    tag: PROFILE,
    profile: PROFILE,
    startedAt,
    elapsedMs: initMs + embedMs + createMs,
    params: { balls: cfg.balls, budget: cfg.budget, sig: cfg.sig, startTopN: cfg.startTopN },
    dim,
    cache: emb.cachePath,
  });
  paths.writeJson(run.resultPath, result);

  return { run, result };
}

// ---------- 主流程 ----------

(async () => {
  const matrix = LADDER
    ? [50, 500, 5000, 50000, 100000]
        .filter((n) => n >= SKIP)
        .map((n) => ({
          balls: n,
          budget: +arg('budget', 2.5),
          sig: +arg('sig', 1.0),
          startTopN: +arg('top', 5),
        }))
    : BUDGETS
      ? BUDGETS.map((b) => ({
          balls: +arg('balls', 5000),
          budget: b,
          sig: +arg('sig', 1.0),
          startTopN: TOPS ? TOPS[0] : +arg('top', 5),
        }))
      : TOPS
        ? TOPS.map((t) => ({
            balls: +arg('balls', 5000),
            budget: +arg('budget', 2.5),
            sig: +arg('sig', 1.0),
            startTopN: t,
          }))
        : [
            {
              balls: +arg('balls', 100),
              budget: +arg('budget', 2.5),
              sig: +arg('sig', 1.0),
              startTopN: +arg('top', 5),
            },
          ];

  const summary = [];
  for (const cfg of matrix) {
    process.stdout.write(`球数=${cfg.balls} budget=${cfg.budget} top=${cfg.startTopN} ... `);
    try {
      const { run, result: r } = await runOne(cfg);
      summary.push({ run: paths.rel(run.dir), ...r, rows: undefined });
      console.log(
        `dim=${r.dim} 管=${r.pipes} 起始=${r.avgStart} 点亮=${r.avgLit} 被挡=${r.avgBlocked} ` +
          `注入=${r.avgBlocks} 命中率=${r.domainHitRate} 延迟=${r.avgRecallMs}ms | ` +
          `嵌入=${r.embedMs}ms(${r.embedMissing}/${r.embedUnique}) 建图=${r.createMs}ms db=${r.dbSizeMB}MB`,
      );
      console.log(`  产物：${paths.rel(run.dir)}`);
    } catch (e) {
      console.log(`FAILED: ${e.message}`);
      summary.push({ ...cfg, error: e.message });
    }
  }

  console.log('\n=== 汇总 ===');
  console.log('球数 | 管道 | 起始 | 点亮 | 被挡 | 强化 | 注入 | token | 延迟 | 命中率 | db');
  for (const r of summary) {
    if (r.error) {
      console.log(`${r.balls} | ERROR ${r.error}`);
      continue;
    }
    console.log(
      `${String(r.balls).padStart(6)} | ${String(r.pipes).padStart(6)} | ${String(r.avgStart).padStart(4)} | ` +
        `${String(r.avgLit).padStart(4)} | ${String(r.avgBlocked).padStart(4)} | ${String(r.avgStrengthened).padStart(4)} | ` +
        `${String(r.avgBlocks).padStart(4)} | ${String(r.avgTokens).padStart(5)} | ${String(r.avgRecallMs).padStart(5)}ms | ` +
        `${String(r.domainHitRate).padStart(5)} | ${r.dbSizeMB}MB`,
    );
  }

  // 多组时额外落一份跨组汇总，便于直接对比
  if (summary.length > 1) {
    const p = `${paths.OUT_DIR}/scale-summary-${paths.stamp()}.json`;
    fs.mkdirSync(paths.OUT_DIR, { recursive: true });
    paths.writeJson(p, summary);
    console.log(`\n跨组汇总已写入 ${paths.rel(p)}`);
  }
})().catch((e) => {
  console.error('FATAL', e.message);
  process.exit(1);
});

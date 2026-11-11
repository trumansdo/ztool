// 召回球的语义级相关率测量
//
// 不看「领域命中」这种粗粒度标签，直接算 query 与每个召回球的余弦。
//
// 指标：
//   1. 注入球余弦分布（均值 / 中位 / P10 / P90 / 分档占比）
//   2. 精确率：注入球中落在「全库 Top-20 近邻」内的比例
//   3. 召回率：全库 Top-20 近邻中被注入的比例
//   4. 随机基线：随机 5 球的平均余弦（噪音水位）
//
// 运行：node test/lab/relevance.cjs --profile http --balls 5000 --budget 10 --top 5
//       node test/lab/relevance.cjs --profile http --balls 5000 --inject 5   # 测预算截断
//
// 产物：test/out/<run>/{meta.json, result.json, db/, logs/}

const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const datasetLib = require('../lib/dataset.cjs');
const { cos, mean, percentile, orderDesc } = require('../lib/stats.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};

const PROFILE = arg('profile', 'http');
const DATASET = arg('dataset', 'main');
const BALLS = +arg('balls', 5000);
const BUDGET = +arg('budget', 10);
const TOP = +arg('top', 5);
// 注入条数上限：默认放开（测召回/点亮质量）。设为 N 则测「只注入 N 条」时的实际质量。
const INJECT = +arg('inject', 5000);
const EMBED_CHUNK = +arg('chunk', 64);

(async () => {
  const ds = datasetLib.load(DATASET);
  const startedAt = new Date().toISOString();
  // 目录名必须带 budget / top：只带球数的话，同规模不同预算会互相覆盖，
  // 「一次运行 = 一个自描述目录」就失效了
  const run = paths.newRun(
    'relevance',
    `${PROFILE}-${BALLS}b-b${BUDGET}-t${TOP}-i${INJECT}`,
  );

  // 放宽注入预算：让尽可能多的点亮球都渲染出来，测的是「召回」而不是「预算截断」
  const { api, dim, identity } = await harness.init({
    profile: PROFILE,
    run,
    overrides: {
      total_water_budget: BUDGET,
      start_top_n: TOP,
      token_budget: 1_000_000,
      max_balls_per_inject: INJECT,
      // max_balls_per_inject 只在 BallCount 模式下生效；Token 模式下由 token_budget 截断
      injection_mode: INJECT < 5000 ? 'BallCount' : 'Token',
    },
  });

  const mems = datasetLib.memories(ds, BALLS);
  const emb = await harness.embed(
    mems.map((m) => m.text),
    { identity, dataset: ds, chunk: EMBED_CHUNK },
  );
  const { ids } = await harness.seedGraph(api, mems, emb.vectors, datasetLib.edges(ds, BALLS));

  const idToIdx = new Map(ids.map((id, i) => [id, i]));
  const queryVecs = await harness.embed(
    ds.queries.map((q) => q.q),
    { identity, dataset: ds, chunk: EMBED_CHUNK },
  );

  const perQuery = [];
  for (let qi = 0; qi < ds.queries.length; qi++) {
    const { q, domain } = ds.queries[qi];
    const qv = queryVecs.vectors[qi];

    // 全库余弦 → 真值排序
    const all = mems.map((_, i) => cos(qv, emb.vectors[i]));
    const order = orderDesc(all);
    const gtTop20 = new Set(order.slice(0, 20).map((i) => ids[i]));

    const r = await api.recall(q, 1.0);
    const recalled = [];
    for (const blk of r.blocks) {
      for (const bid of blk.ballIds || []) {
        const idx = idToIdx.get(bid);
        if (idx === undefined) continue;
        recalled.push({
          id: bid,
          cos: all[idx],
          domain: mems[idx].domain,
          text: mems[idx].text,
          tier: blk.confidence,
        });
      }
    }

    const cosines = recalled.map((x) => x.cos);
    const hitTop20 = recalled.filter((x) => gtTop20.has(x.id)).length;
    // 随机基线：确定性抽 5 个球
    const rnd = Array.from({ length: 5 }, (_, k) => all[(qi * 977 + k * 131) % BALLS]);

    perQuery.push({
      query: q,
      domain,
      gtBest: all[order[0]],
      gtTop20Floor: all[order[19]],
      litCount: r.stats.litCount,
      recalledCount: recalled.length,
      meanCos: mean(cosines),
      medianCos: percentile(cosines, 50),
      minCos: cosines.length ? Math.min(...cosines) : null,
      maxCos: cosines.length ? Math.max(...cosines) : null,
      precisionAt20: recalled.length ? hitTop20 / recalled.length : null,
      recallAt20: hitTop20 / 20,
      randomBaseline: mean(rnd),
      recalled,
    });
    process.stdout.write('.');
  }
  console.log('');

  api.shutdown();

  // ---- 汇总 ----
  const allCos = perQuery.flatMap((p) => p.recalled.map((x) => x.cos));
  const buckets = [
    ['>=0.60 强相关', (c) => c >= 0.6],
    ['0.50~0.60 相关', (c) => c >= 0.5 && c < 0.6],
    ['0.40~0.50 弱相关', (c) => c >= 0.4 && c < 0.5],
    ['0.35~0.40 边缘', (c) => c >= 0.35 && c < 0.4],
    ['<0.35 不相关', (c) => c < 0.35],
  ];

  const summary = {
    dataset: ds.id,
    datasetSourceHash: ds.sourceHash.slice(0, 8),
    balls: BALLS,
    dim,
    profile: PROFILE,
    budget: BUDGET,
    topN: TOP,
    inject: INJECT,
    queries: perQuery.length,
    recalledTotal: allCos.length,
    meanCos: +mean(allCos).toFixed(4),
    medianCos: +percentile(allCos, 50).toFixed(4),
    p10: +percentile(allCos, 10).toFixed(4),
    p90: +percentile(allCos, 90).toFixed(4),
    buckets: buckets.map(([label, fn]) => {
      const count = allCos.filter(fn).length;
      return { label, count, ratio: +(count / allCos.length).toFixed(3) };
    }),
    meanPrecisionAt20: +mean(perQuery.map((p) => p.precisionAt20)).toFixed(3),
    meanRecallAt20: +mean(perQuery.map((p) => p.recallAt20)).toFixed(3),
    meanRandomBaseline: +mean(perQuery.map((p) => p.randomBaseline)).toFixed(4),
    meanGtBest: +mean(perQuery.map((p) => p.gtBest)).toFixed(4),
    meanLit: +mean(perQuery.map((p) => p.litCount)).toFixed(1),
    meanRecalled: +mean(perQuery.map((p) => p.recalledCount)).toFixed(1),
  };

  console.log('=== 汇总 ===');
  console.log(JSON.stringify(summary, null, 2));
  console.log('\n=== 分档 ===');
  for (const b of summary.buckets) {
    console.log(`${b.label.padEnd(16)} ${String(b.count).padStart(5)}  ${(b.ratio * 100).toFixed(1)}%`);
  }
  console.log('\n=== 每查询 ===');
  console.log('查询 | 点亮 | 召回 | 均值cos | 中位 | 精确率@20 | 召回率@20 | 随机基线 | 真值最佳');
  for (const p of perQuery) {
    console.log(
      `${p.query.slice(0, 18).padEnd(18)} | ${String(p.litCount).padStart(3)} | ${String(p.recalledCount).padStart(3)} | ` +
        `${p.meanCos.toFixed(3)} | ${p.medianCos.toFixed(3)} | ${p.precisionAt20.toFixed(2)} | ` +
        `${p.recallAt20.toFixed(2)} | ${p.randomBaseline.toFixed(3)} | ${p.gtBest.toFixed(3)}`,
    );
  }

  paths.writeJson(run.resultPath, { summary, perQuery });
  harness.writeMeta(run, {
    script: 'relevance',
    tag: PROFILE,
    profile: PROFILE,
    startedAt,
    params: { balls: BALLS, budget: BUDGET, top: TOP, inject: INJECT },
    dim,
    cache: emb.cachePath,
  });

  console.log(`\n产物：${paths.rel(run.dir)}`);
})().catch((e) => {
  console.error('FATAL', e.message);
  process.exit(1);
});

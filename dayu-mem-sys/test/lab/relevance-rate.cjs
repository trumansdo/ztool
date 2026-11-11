// 相关率（召回率）：|召回 ∩ 相关| / |相关| —— 离线分析，不建库、不调 API
//
// 「相关」不再用人为的 Top-20，而是按余弦阈值定义：
//   相关集 = 全库中 cos(query, ball) >= 阈值的所有球
// 阈值取 0.4 / 0.5 / 0.6 三档（实测：同领域 0.64~0.85，跨领域 0.19~0.31，随机基线 0.37）
//
// 同时给出去重版（语料 375 基础句铺成 N 球，每句约 N/375 份副本）。
//
// 运行：
//   node test/lab/relevance-rate.cjs                    # 取最近一次 relevance 运行
//   node test/lab/relevance-rate.cjs --run test/out/20261110-2230-relevance-http-5000b-i5000

const path = require('path');

const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const { baseOf } = require('../lib/corpus.cjs');
const datasetLib = require('../lib/dataset.cjs');
const cacheLib = require('../lib/cache.cjs');
const { cos, mean, hashOf, pctStr } = require('../lib/stats.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};
const THRESHOLDS = [0.4, 0.5, 0.6];

/** 定位运行目录：显式 --run，否则取最近一次 relevance 运行 */
function resolveRun() {
  const explicit = arg('run', null);
  if (explicit) return path.resolve(paths.REPO_ROOT, explicit);
  const latest = paths.listRuns().find((r) => r.meta.script === 'relevance');
  if (!latest) throw new Error('没有找到 relevance 运行目录，请先跑 lab/relevance.cjs');
  return latest.dir;
}

function main() {
  const runDir = resolveRun();
  const meta = harness.readMeta(runDir);
  const result = paths.readJson(path.join(runDir, 'result.json'));
  if (!result) throw new Error(`${runDir} 缺少 result.json`);

  const cache = cacheLib.load(path.resolve(paths.REPO_ROOT, meta.cache));
  if (!cache.size) throw new Error(`缺少向量缓存 ${meta.cache}`);

  const BALLS = result.summary.balls;
  const mems = datasetLib.memories(datasetLib.load(result.summary.dataset ?? 'main'), BALLS);
  const vectors = mems.map((m) => cache.get(hashOf(m.text)));

// 基础句 → 副本下标
const baseIndex = new Map();
mems.forEach((m, i) => {
  const b = baseOf(m.text);
  if (!baseIndex.has(b)) baseIndex.set(b, []);
  baseIndex.get(b).push(i);
});

const rows = [];
for (const p of result.perQuery) {
  const qv = cache.get(hashOf(p.query));
  if (!qv) throw new Error(`缓存缺查询向量「${p.query}」`);
  const all = vectors.map((v) => cos(qv, v));

  const recalledIds = new Set(p.recalled.map((x) => x.id));
  const recalledBases = new Set(p.recalled.map((x) => baseOf(x.text)));

  const row = {
    query: p.query,
    lit: p.litCount,
    recalled: p.recalled.length,
    distinctRecalled: recalledBases.size,
  };

  for (const th of THRESHOLDS) {
    // 相关集：球级
    const relevant = [];
    for (let i = 0; i < BALLS; i++) if (all[i] >= th) relevant.push(i);
    // 相关集：去重基础句级
    const relevantBases = new Set();
    for (const [text, idxs] of baseIndex) {
      if (Math.max(...idxs.map((i) => all[i])) >= th) relevantBases.add(text);
    }

    // 球 id 从 1 开始顺序分配（每次运行都是全新的库，故 idx = id - 1）
    const hitBalls = relevant.filter((i) => recalledIds.has(i + 1)).length;
    const hitBases = [...recalledBases].filter((b) => relevantBases.has(b)).length;

    row[`relevant@${th}`] = relevant.length;
    row[`recall@${th}`] = relevant.length ? hitBalls / relevant.length : null;
    row[`relevantBase@${th}`] = relevantBases.size;
    row[`recallBase@${th}`] = relevantBases.size ? hitBases / relevantBases.size : null;
  }
  rows.push(row);
}

const f2 = (x) => (x === null ? 'n/a' : `${(x * 100).toFixed(1)}%`);

console.log(`=== 相关率（召回 ∩ 相关 / 相关）  运行：${path.basename(runDir)} ===`);
console.log('查询 | 相关@0.4 | 相关@0.5 | 相关@0.6 | 召回 | 相关率@0.4 | @0.5 | @0.6');
for (const r of rows) {
  console.log(
    `${r.query.slice(0, 16).padEnd(16)} | ${String(r['relevant@0.4']).padStart(5)} | ` +
      `${String(r['relevant@0.5']).padStart(5)} | ${String(r['relevant@0.6']).padStart(5)} | ` +
      `${String(r.recalled).padStart(3)} | ${f2(r['recall@0.4']).padStart(6)} | ` +
      `${f2(r['recall@0.5']).padStart(6)} | ${f2(r['recall@0.6']).padStart(6)}`,
  );
}

console.log('\n=== 球级平均 ===');
for (const th of THRESHOLDS) {
  console.log(
    `阈值 ${th}: 平均相关球数 ${mean(rows.map((r) => r[`relevant@${th}`])).toFixed(0)}，` +
      `平均召回率 ${f2(mean(rows.map((r) => r[`recall@${th}`]).filter((x) => x !== null)))}`,
  );
}

console.log('\n=== 去重（按基础句）平均 ===');
for (const th of THRESHOLDS) {
  console.log(
    `阈值 ${th}: 平均相关句数 ${mean(rows.map((r) => r[`relevantBase@${th}`])).toFixed(0)}，` +
      `平均召回率 ${f2(mean(rows.map((r) => r[`recallBase@${th}`]).filter((x) => x !== null)))}`,
  );
}

// 强线索 vs 普通
const bestOf = new Map(result.perQuery.map((p) => [p.query, p.gtBest]));
const isStrong = (r) => (bestOf.get(r.query) ?? 0) >= 0.8;

console.log('\n=== 分组（强线索 = 真值最佳余弦 ≥ 0.8）===');
for (const [name, g] of [
  ['强线索', rows.filter(isStrong)],
  ['普通', rows.filter((r) => !isStrong(r))],
]) {
  if (!g.length) {
    console.log(`${name.padEnd(8)} 查询 0`);
    continue;
  }
  console.log(
    `${name.padEnd(8)} 查询 ${String(g.length).padStart(2)} | 平均点亮 ${mean(g.map((r) => r.lit)).toFixed(1)} | ` +
      `相关球@0.4 ${mean(g.map((r) => r['relevant@0.4'])).toFixed(0)} | ` +
      `召回率@0.4 ${f2(mean(g.map((r) => r['recall@0.4'])))} | ` +
      `@0.5 ${f2(mean(g.map((r) => r['recall@0.5'])))} | @0.6 ${f2(mean(g.map((r) => r['recall@0.6'])))}`,
  );
}

const outPath = path.join(runDir, 'relevance-rate.json');
  paths.writeJson(outPath, rows);
  console.log(`\n明细已写入 ${paths.rel(outPath)}`);
}

try {
  main();
} catch (e) {
  console.error('FAILED', e.message);
  process.exit(1);
}

// 跨规模准确率对照（默认 4B vs 0.6B）
//
// 为什么不能用 precision@K / recall@K 比跨规模：
//   语料是 375 条基础句铺到 N 个球，N 越大重复越多。
//   N=5000 时 GT Top-20 是同一句的 20 份副本，注入副本即「全中」，
//   纯度被人为抬高（4B 在 5000 的纯度 0.940 反而高于 2000 的 0.807）。
//   所以纯度只能同规模比，不能跨规模比。
//
// 跨规模可比的三个量：
//   1. 同领域率   —— 注入集里与查询同领域的比例（人工定义，与模型无关，与重复无关）
//   2. 余弦裕度   —— 注入集平均余弦 − 全库平均余弦（去掉各模型的尺度差）
//   3. 纯度       —— 仅同规模内比
//
// 运行：node test/lab/relevance-scale.cjs            # 扫描 test/out/ 下所有 relevance 运行

const fs = require('fs');
const path = require('path');

const paths = require('../lib/paths.cjs');
const { mean, signTest, pctStr } = require('../lib/stats.cjs');

/** 收集所有 relevance 运行：tag → [{n, inject, dim, ...}] */
function collect() {
  const byTag = new Map();
  for (const run of paths.listRuns()) {
    if (run.meta.script !== 'relevance') continue;
    const data = paths.readJson(run.resultPath);
    if (!data) continue;
    const { summary, perQuery } = data;

    const domainPrec = mean(
      perQuery.map((p) =>
        p.recalled.length ? p.recalled.filter((r) => r.domain === p.domain).length / p.recalled.length : 0,
      ),
    );

    const row = {
      dir: paths.rel(run.dir),
      tag: run.meta.profile,
      n: summary.balls,
      inject: summary.inject,
      dim: summary.dim,
      // 注入集平均余弦 − 全库平均余弦（线性尺度，跨模型可比）
      cosMargin: summary.meanCos - summary.meanRandomBaseline,
      cosRatio: summary.meanCos / summary.meanRandomBaseline,
      meanCos: summary.meanCos,
      baseline: summary.meanRandomBaseline,
      domainPrec,
      purity: summary.meanPrecisionAt20,
      meanLit: summary.meanLit,
      meanRecalled: summary.meanRecalled,
    };

    const key = `${row.tag}|${row.inject}`;
    if (!byTag.has(key)) byTag.set(key, []);
    byTag.get(key).push(row);
  }
  for (const rows of byTag.values()) rows.sort((a, b) => a.n - b.n);
  return byTag;
}

const byTag = collect();
if (byTag.size === 0) {
  console.error('没有找到任何 relevance 运行，请先跑 lab/relevance.cjs');
  process.exit(1);
}

const keys = [...byTag.keys()];
const tags = keys.map((k) => k.split('|')[0]);
const ns = [...new Set(keys.flatMap((k) => byTag.get(k).map((r) => r.n)))].sort((a, b) => a - b);

console.log('=== 规模对照 ===');
for (const k of keys) {
  console.log(`${k.padEnd(16)} dim=${byTag.get(k)[0].dim}  运行数=${byTag.get(k).length}`);
}

const table = (title, key, fmt, deltaFmt) => {
  console.log(`\n=== ${title} ===`);
  console.log('N'.padStart(7) + keys.map((k) => k.padStart(16)).join('') + '   差值');
  for (const n of ns) {
    const vals = keys.map((k) => byTag.get(k).find((r) => r.n === n));
    if (vals.some((v) => !v)) continue;
    const cells = vals.map((v) => fmt(v[key]).padStart(16));
    const d = vals.length === 2 ? vals[1][key] - vals[0][key] : null;
    console.log(String(n).padStart(7) + cells.join('') + (d === null ? '' : `  ${deltaFmt(d).padStart(9)}`));
  }
};

const pp = (d) => `${d >= 0 ? '+' : ''}${(d * 100).toFixed(1)}pp`;
const f3 = (d) => `${d >= 0 ? '+' : ''}${d.toFixed(3)}`;
const f1 = (d) => `${d >= 0 ? '+' : ''}${d.toFixed(1)}`;

table('同领域率（注入集中与查询同领域的比例，人工定义）', 'domainPrec', pctStr, pp);
table('余弦裕度（注入集平均余弦 − 全库平均余弦，线性尺度）', 'cosMargin', (x) => x.toFixed(3), f3);
table('余弦/基线（比值口径，参考）', 'cosRatio', (x) => x.toFixed(3), f3);
table('纯度（命中该模型自身 Top-20 的比例，仅同规模可比）', 'purity', pctStr, pp);
table('平均点亮球数', 'meanLit', (x) => x.toFixed(1), f1);

// ---------- 配对比较：每个规模上逐查询对比同领域率 ----------

const domPrec = (p) =>
  p.recalled.length ? p.recalled.filter((r) => r.domain === p.domain).length / p.recalled.length : 0;

if (keys.length === 2) {
  for (const [label, fn] of [
    ['同领域率（人工口径）', domPrec],
    ['纯度（模型自身 Top-20）', (p) => p.precisionAt20 ?? 0],
  ]) {
    console.log(`\n=== 配对比较：${label}（逐查询）===`);
    console.log('N'.padStart(7) + '   胜   平   负    p');
    for (const n of ns) {
      const ra = paths.readJson(byTag.get(keys[0]).find((r) => r.n === n).dir + '/result.json');
      const rb = paths.readJson(byTag.get(keys[1]).find((r) => r.n === n).dir + '/result.json');
      if (!ra || !rb) continue;
      let win = 0;
      let tie = 0;
      let lose = 0;
      for (let i = 0; i < ra.perQuery.length; i++) {
        const x = fn(ra.perQuery[i]);
        const y = fn(rb.perQuery[i]);
        if (Math.abs(x - y) < 1e-9) tie++;
        else if (x > y) win++;
        else lose++;
      }
      console.log(
        String(n).padStart(7) +
          `${String(win).padStart(5)}${String(tie).padStart(5)}${String(lose).padStart(5)}   ` +
          signTest(win, lose).toFixed(3),
      );
    }
  }
}

const outPath = path.join(paths.OUT_DIR, `relevance-scale-${paths.stamp()}.json`);
paths.writeJson(outPath, Object.fromEntries(byTag));
console.log(`\n明细已写入 ${paths.rel(outPath)}`);

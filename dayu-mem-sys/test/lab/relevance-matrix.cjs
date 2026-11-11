// 规模 × 配置矩阵：批量跑 relevance.cjs，汇总成一张表
//
// 每个组合起一个子进程：napi 的 init 一个进程只能做一次，同进程连跑多组会
// 踩 shutdown 后的状态。子进程也保证「一组崩了不影响其他组」。
//
// 用法：
//   node test/lab/relevance-matrix.cjs --profile lms --balls 500,2000,5000,20000 --budgets 1,2.5,10
//   node test/lab/relevance-matrix.cjs --profile lms --balls 5000 --tops 3,5,8
//
// 产出：终端一张表 + `test/out/<时间戳>-matrix-<profile>/matrix.json`
//
// 指标口径（与 relevance.cjs 一致）：
//   precision@20  系统拿出来的球里，有多少落在「全库余弦真值 top-20」内 —— 准确率
//   recall@20     命中数 / 20 —— 召回率
//   meanCos       拿出来那批球与 query 的平均余弦 —— 绝对语义相关度
//   裕度          meanCos − 随机 5 球均值 —— 证明不是瞎猫碰上死耗子
//   cos@K         每个 query 把拿到的球按余弦降序取前 K 求平均，不足 K 条补 0
//
// 为什么必须有 cos@K：meanCos 会被「注入条数」污染。budget 小 → 只注入起始球
// → meanCos 天然高，但那是「拿得少」不是「拿得准」。cos@K 把两者分开：不足 K
// 条补 0，所以「只给 3 条」不会因为少而占便宜。

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const paths = require('../lib/paths.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};
const listArg = (name, def) =>
  arg(name, def)
    .split(',')
    .map(Number)
    .filter((x) => Number.isFinite(x));

const PROFILE = arg('profile', 'lms');
const BALLS = listArg('balls', '500,2000,5000,20000');
const BUDGETS = listArg('budgets', '1,2.5,10');
const TOPS = listArg('tops', '5');
const INJECT = arg('inject', '5000');
const TIMEOUT_MS = +arg('timeout', 30 * 60 * 1000);

/** 排序质量：把该 query 拿到的球按余弦降序取前 K，不足补 0 */
function cosAtK(dir, K) {
  const result = JSON.parse(
    fs.readFileSync(path.resolve(paths.REPO_ROOT, dir, 'result.json'), 'utf8'),
  );
  const per = result.perQuery.map((p) => {
    const cos = p.recalled.map((x) => x.cos).sort((a, b) => b - a);
    let sum = 0;
    for (let i = 0; i < K; i++) sum += cos[i] ?? 0;
    return sum / K;
  });
  return per.reduce((a, b) => a + b, 0) / per.length;
}

/** 一轮 = 一个 relevance.cjs 子进程 */
function runOne({ balls, budget, top }) {
  const t0 = Date.now();
  const r = spawnSync(
    process.execPath,
    [
      path.join(__dirname, 'relevance.cjs'),
      '--profile', PROFILE,
      '--balls', String(balls),
      '--budget', String(budget),
      '--top', String(top),
      '--inject', String(INJECT),
    ],
    { encoding: 'utf8', timeout: TIMEOUT_MS, maxBuffer: 64 * 1024 * 1024 },
  );

  if (r.status !== 0) {
    return {
      balls, budget, top,
      error: (r.stderr || r.stdout || '').trim().split('\n').slice(-3).join(' | ') || `退出码 ${r.status}`,
      ms: Date.now() - t0,
    };
  }

  // 产物目录从 stdout 精确取，不靠「out/ 下最新的 relevance 目录」猜
  const m = /产物：(\S+)/.exec(r.stdout);
  if (!m) return { balls, budget, top, error: '没找到产物目录', ms: Date.now() - t0 };

  const result = JSON.parse(fs.readFileSync(path.resolve(paths.REPO_ROOT, m[1], 'result.json'), 'utf8'));
  const s = result.summary;

  return {
    balls, budget, top,
    dir: m[1],
    meanCos: s.meanCos,
    medianCos: s.medianCos,
    p10: s.p10,
    precisionAt20: s.meanPrecisionAt20,
    recallAt20: s.meanRecallAt20,
    baseline: s.meanRandomBaseline,
    gtBest: s.meanGtBest,
    lit: s.meanLit,
    recalled: s.meanRecalled,
    strongRatio: s.buckets?.find((b) => b.label.startsWith('>=0.60'))?.ratio ?? null,
    ms: Date.now() - t0,
  };
}

// ---------- 主流程 ----------

// --analyze <matrix.json>：不重跑，只对已有结果重算汇总（补指标时用）
const ANALYZE = arg('analyze', null);

let rows;
let run;

if (ANALYZE) {
  const prev = JSON.parse(fs.readFileSync(ANALYZE, 'utf8'));
  rows = prev.rows;
  run = { dir: path.dirname(ANALYZE) };
  console.log(`复用 ${ANALYZE}（${rows.length} 组，profile=${prev.profile}）\n`);
} else {
  const combos = [];
  for (const balls of BALLS) for (const budget of BUDGETS) for (const top of TOPS) {
    combos.push({ balls, budget, top });
  }

  run = paths.newRun('matrix', `${PROFILE}-${combos.length}组`);
  rows = [];

  console.log(`profile=${PROFILE}  组合 ${combos.length} 组  注入上限 ${INJECT}\n`);

  combos.forEach((c, i) => {
    process.stdout.write(
      `[${String(i + 1).padStart(2)}/${combos.length}] ${c.balls}球 b${c.budget} t${c.top} … `,
    );
    const row = runOne(c);
    rows.push(row);
    if (row.error) {
      console.log(`失败：${row.error}`);
    } else {
      console.log(
        `cos ${row.meanCos} / p@20 ${row.precisionAt20} / r@20 ${row.recallAt20} / 点亮 ${row.lit} / ${(row.ms / 1000).toFixed(0)}s`,
      );
    }
  });
}

// ---------- 汇总表 ----------

const ok = rows.filter((r) => !r.error);
const fmt = (v, d = 3) => (v === null || v === undefined ? '  -  ' : v.toFixed(d).padStart(6));

// 排序质量：与注入条数无关，跨 budget / 跨规模唯一可比的「准不准」
for (const r of ok) {
  r.cosAt5 = cosAtK(r.dir, 5);
  r.cosAt10 = cosAtK(r.dir, 10);
}

console.log(`\n=== 矩阵（${PROFILE}）===`);
console.log(
  '规模'.padStart(6) + ' | ' + 'budget'.padStart(6) + ' | ' + 'top'.padStart(3) + ' | ' +
  'cos@5'.padStart(6) + ' | ' + 'cos@10'.padStart(6) + ' | ' + 'meanCos'.padStart(7) + ' | ' +
  'p@20'.padStart(6) + ' | ' + 'r@20'.padStart(6) + ' | ' + '裕度'.padStart(6) + ' | ' +
  '点亮'.padStart(5) + ' | ' + '注入'.padStart(5),
);
console.log('-'.repeat(96));

for (const r of ok) {
  const margin = r.meanCos - r.baseline;
  console.log(
    String(r.balls).padStart(6) + ' | ' +
    String(r.budget).padStart(6) + ' | ' +
    String(r.top).padStart(3) + ' | ' +
    fmt(r.cosAt5) + ' | ' +
    fmt(r.cosAt10) + ' | ' +
    fmt(r.meanCos) + ' | ' +
    fmt(r.precisionAt20) + ' | ' +
    fmt(r.recallAt20) + ' | ' +
    (margin >= 0 ? '+' : '') + fmt(margin) + ' | ' +
    String(r.lit).padStart(5) + ' | ' +
    String(r.recalled).padStart(5),
  );
}

// 每个规模的最优 budget（按 cos@5：与注入条数无关，才是真的「更准」）
console.log('\n=== 每个规模的最优 budget（按 cos@5）===');
for (const balls of [...new Set(ok.map((r) => r.balls))].sort((a, b) => a - b)) {
  const group = ok.filter((r) => r.balls === balls);
  const best = group.reduce((a, b) => (b.cosAt5 > a.cosAt5 ? b : a));
  console.log(
    `${String(balls).padStart(6)} 球 → budget ${best.budget}（cos@5 ${best.cosAt5.toFixed(3)}，` +
      `cos@10 ${best.cosAt10.toFixed(3)}，注入 ${best.recalled} 条，点亮 ${best.lit}）`,
  );
}

// 同一 budget 下，规模对准确率的影响
console.log('\n=== 规模效应（同 budget 下 cos@5 的变化）===');
for (const budget of [...new Set(ok.map((r) => r.budget))].sort((a, b) => a - b)) {
  const group = ok.filter((r) => r.budget === budget).sort((a, b) => a.balls - b.balls);
  console.log(
    `budget ${String(budget).padEnd(4)} ` +
      group.map((r) => `${r.balls}:${r.cosAt5.toFixed(3)}`).join('  →  '),
  );
}

const out = { profile: PROFILE, inject: +INJECT, at: new Date().toISOString(), rows };
paths.writeJson(path.join(run.dir, 'matrix.json'), out);
console.log(`\n矩阵数据：${run.dir}/matrix.json`);

if (rows.some((r) => r.error)) {
  console.log('\n有组合失败：');
  for (const r of rows.filter((x) => x.error)) {
    console.log(`  ${r.balls}球 b${r.budget} t${r.top}：${r.error}`);
  }
  process.exit(1);
}

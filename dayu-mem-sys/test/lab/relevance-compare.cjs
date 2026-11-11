// 跨嵌入模型对照：只用「与相似度尺度无关」的指标
//
// 为什么不能直接用固定余弦阈值比：
//   每个模型的相似度分布尺度不同。实测随机基线 4B=0.3669、0.6B=0.3321，
//   同一个 0.6 阈值对 0.6B 相当于更高的相对门槛，相关集大小差 1.5 倍。
//   分母不同，比值不可比。
//
// 因此这里只用两类尺度无关指标：
//   1. recall@K / precision@K —— 相关集 = 全库 cos 最高的 K 个（K 固定）
//   2. 阈值按各模型自身的随机基线平移 —— 相关集 = cos >= baseline + Δ
//
// 运行：
//   node test/lab/relevance-compare.cjs                     # 自动取各 profile 最近一次
//   node test/lab/relevance-compare.cjs --runs http=test/out/xxx,lms=test/out/yyy

const path = require('path');

const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const { expandCorpus } = require('../lib/corpus.cjs');
const { cos, mean, orderDesc, hashOf, signTest, pctStr } = require('../lib/stats.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};

const KS = [5, 10, 20, 50, 100];
const DELTAS = [0.05, 0.1, 0.15, 0.2];

/** 解析 --runs：显式列表，或各 profile 最近一次 relevance 运行 */
function resolveRuns() {
  const explicit = arg('runs', null);
  if (explicit) {
    return explicit.split(',').map((s) => {
      const [tag, dir] = s.split('=');
      return { tag, dir: path.resolve(paths.REPO_ROOT, dir) };
    });
  }
  const byProfile = new Map();
  for (const r of paths.listRuns()) {
    if (r.meta.script !== 'relevance') continue;
    if (!byProfile.has(r.meta.profile)) byProfile.set(r.meta.profile, r.dir);
  }
  if (byProfile.size < 2) {
    throw new Error(
      `需要至少两个 profile 的 relevance 运行，当前只有 ${[...byProfile.keys()].join(', ') || '无'}`,
    );
  }
  return [...byProfile].map(([tag, dir]) => ({ tag, dir }));
}

function loadTag({ tag, dir }) {
  const meta = harness.readMeta(dir);
  const result = paths.readJson(path.join(dir, 'result.json'));
  if (!result) throw new Error(`${dir} 缺少 result.json`);
  const cache = paths.readJson(path.resolve(paths.REPO_ROOT, meta.cache));
  if (!cache) throw new Error(`${tag}：缺少向量缓存 ${meta.cache}`);

  const mems = expandCorpus(result.summary.balls);
  const vecOf = (t) => {
    const v = cache[hashOf(t)];
    if (!v) throw new Error(`${tag}：缓存缺向量「${t.slice(0, 20)}」`);
    return v;
  };
  return { tag, dir, result, mems, ballVecs: mems.map((m) => vecOf(m.text)), vecOf };
}

function analyse(t) {
  const { tag, result, ballVecs, vecOf } = t;
  const perK = new Map(KS.map((k) => [k, { hit: 0, gt: 0, rec: 0 }]));
  const perDelta = new Map(DELTAS.map((d) => [d, { hit: 0, gt: 0, rec: 0 }]));
  const baselines = [];

  for (const pq of result.perQuery) {
    const qv = vecOf(pq.query);
    const sims = ballVecs.map((bv) => cos(qv, bv));
    const order = orderDesc(sims);
    // 召回集：result 里记录的 id → 语料下标（球 id 从 1 开始顺序分配）
    const recalledIdx = new Set(pq.recalled.map((r) => r.id - 1));
    const baseline = mean(sims);
    baselines.push(baseline);

    for (const k of KS) {
      const gt = new Set(order.slice(0, k));
      const agg = perK.get(k);
      agg.hit += [...recalledIdx].filter((i) => gt.has(i)).length;
      agg.gt += k;
      agg.rec += recalledIdx.size;
    }
    for (const d of DELTAS) {
      const thr = baseline + d;
      const agg = perDelta.get(d);
      agg.hit += [...recalledIdx].filter((i) => sims[i] >= thr).length;
      agg.gt += sims.filter((s) => s >= thr).length;
      agg.rec += recalledIdx.size;
    }
  }

  const n = result.perQuery.length;
  return {
    tag,
    dir: paths.rel(t.dir),
    dim: result.summary.dim,
    meanBaseline: mean(baselines),
    perK: [...perK].map(([k, a]) => ({
      k,
      recall: a.hit / a.gt,
      precision: a.hit / a.rec,
      gtPerQuery: a.gt / n,
    })),
    perDelta: [...DELTAS].map((d) => {
      const a = perDelta.get(d);
      return { delta: d, recall: a.gt ? a.hit / a.gt : null, gtPerQuery: a.gt / n };
    }),
  };
}

// ---------- 主流程 ----------

function main() {
  const inputs = resolveRuns();
  const rows = inputs.map((i) => analyse(loadTag(i)));
  const tags = rows.map((r) => r.tag);

  console.log('=== 模型 ===');
  for (const r of rows) {
    console.log(`${r.tag.padEnd(6)} dim=${String(r.dim).padEnd(5)} 随机基线=${r.meanBaseline.toFixed(4)}   ${r.dir}`);
  }

  console.log('\n=== recall@K（相关集 = 全库 cos 最高的 K 个，尺度无关）===');
  console.log('K'.padStart(5) + tags.map((t) => t.padStart(12)).join(''));
  for (const k of KS) {
    console.log(
      String(k).padStart(5) +
        rows.map((r) => pctStr(r.perK.find((x) => x.k === k).recall).padStart(12)).join(''),
    );
  }

  console.log('\n=== precision@K（召回集中落在 Top-K 的比例）===');
  console.log('K'.padStart(5) + tags.map((t) => t.padStart(12)).join(''));
  for (const k of KS) {
    console.log(
      String(k).padStart(5) +
        rows.map((r) => pctStr(r.perK.find((x) => x.k === k).precision).padStart(12)).join(''),
    );
  }

  console.log('\n=== 阈值按各模型随机基线平移（相关集 = cos >= baseline + Δ）===');
  console.log('Δ'.padStart(6) + tags.map((t) => t.padStart(12)).join('') + '   （相关集均值）');
  for (const d of DELTAS) {
    console.log(
      `+${d.toFixed(2)}`.padStart(6) +
        rows.map((r) => pctStr(r.perDelta.find((x) => x.delta === d).recall).padStart(12)).join('') +
        '   ' +
        rows.map((r) => r.perDelta.find((x) => x.delta === d).gtPerQuery.toFixed(1)).join(' / '),
    );
  }

  // ---------- 配对比较（逐查询，n=30，比均值更有说服力）----------

  if (rows.length === 2) {
    const [a, b] = inputs;
    const ra = a.result.perQuery;
    const rb = b.result.perQuery;
    console.log(`\n=== 配对比较（${a.tag} vs ${b.tag}，逐查询）===`);
    for (const key of ['recallAt20', 'precisionAt20', 'meanCos']) {
      let win = 0;
      let tie = 0;
      let lose = 0;
      let sumA = 0;
      let sumB = 0;
      for (let i = 0; i < ra.length; i++) {
        const x = ra[i][key];
        const y = rb[i][key];
        sumA += x;
        sumB += y;
        if (Math.abs(x - y) < 1e-9) tie++;
        else if (x > y) win++;
        else lose++;
      }
      console.log(
        `${key.padEnd(14)} ${a.tag} 胜 ${String(win).padStart(2)} / 平 ${String(tie).padStart(2)} / ` +
          `负 ${String(lose).padStart(2)}   均值 ${(sumA / ra.length).toFixed(4)} vs ${(sumB / rb.length).toFixed(4)}`,
      );
    }
    let w = 0;
    let l = 0;
    for (let i = 0; i < ra.length; i++) {
      if (ra[i].recallAt20 > rb[i].recallAt20) w++;
      else if (ra[i].recallAt20 < rb[i].recallAt20) l++;
    }
    console.log(`recall@20 符号检验 p = ${signTest(w, l).toFixed(3)}（n=${w + l}）`);
  }

  const outPath = path.join(paths.OUT_DIR, `relevance-compare-${paths.stamp()}.json`);
  paths.writeJson(outPath, rows);
  console.log(`\n明细已写入 ${paths.rel(outPath)}`);
}

try {
  main();
} catch (e) {
  console.error('FAILED', e.message);
  process.exit(1);
}

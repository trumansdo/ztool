// 跨嵌入模型对照：只用「与相似度尺度无关」的指标
//
// 为什么不能直接用固定余弦阈值比：
//   每个模型的相似度分布尺度不同。实测随机基线 4B=0.3669、0.6B=0.3321，
//   同一个 0.6 阈值对 0.6B 相当于更高的相对门槛，相关集大小差 1.5 倍
//   （@0.5：39 句 vs 25 句）。分母不同，比值不可比。
//
// 因此这里只用两类尺度无关指标：
//   1. recall@K / precision@K —— 相关集 = 全库 cos 最高的 K 个（K 固定）
//   2. 阈值按各模型自身的随机基线平移 —— 相关集 = cos >= baseline + Δ
//
// 运行：node test/relevance-compare.cjs --tags http,lms

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { CORPUS, QUERIES } = require(path.join(__dirname, 'corpus.cjs'));

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};
const TAGS = arg('tags', 'http,lms').split(',');
const KS = [5, 10, 20, 50, 100];
const DELTAS = [0.05, 0.1, 0.15, 0.2];

const DOMAINS = Object.keys(CORPUS);
const LEADINS = [
  '', '记一下：', '补充一点：', '昨天想到：', '和同事聊到：', '看文章时记的：', '复盘：',
  '再确认一下：', '突然想起来：', '周会上提过：', '整理笔记：', '踩坑记录：', '值得记：',
  '睡前回顾：', '路上想到：', '技术分享里听到：', '文档里看到：', '实践下来：', '重新整理：', '给自己提醒：',
];

function expandCorpus(n) {
  const out = [];
  const maxPerDomain = Math.max(...DOMAINS.map((d) => CORPUS[d].length));
  for (let r = 0; r < maxPerDomain && out.length < n; r++) {
    for (const domain of DOMAINS) {
      const text = CORPUS[domain][r];
      if (text === undefined) continue;
      out.push({ text, domain });
      if (out.length >= n) break;
    }
  }
  let i = 0;
  const baseCount = out.length;
  while (out.length < n) {
    const base = out[i % baseCount];
    const lead = LEADINS[(1 + Math.floor(i / baseCount)) % LEADINS.length];
    out.push({ text: `${lead}${base.text}`, domain: base.domain });
    i++;
  }
  return out;
}

const cos = (a, b) => {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += a[i] * b[i];
  return s;
};
const mean = (arr) => (arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : 0);
const pct = (x) => `${(x * 100).toFixed(1)}%`;

function loadTag(tag) {
  const cachePath = path.join(__dirname, `embed-cache-${tag}.json`);
  const resultPath = path.join(__dirname, `relevance-result-${tag}.json`);
  if (!fs.existsSync(cachePath) || !fs.existsSync(resultPath)) {
    throw new Error(`缺少 ${tag} 的缓存或结果：${cachePath} / ${resultPath}`);
  }
  const cache = JSON.parse(fs.readFileSync(cachePath, 'utf8'));
  const result = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
  const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');
  const vecOf = (t) => cache[hashOf(t)];
  const balls = result.summary.balls;
  const mems = expandCorpus(balls);
  const ballVecs = mems.map((m) => vecOf(m.text));
  return { tag, result, mems, ballVecs, vecOf };
}

function analyse(t) {
  const { tag, result, mems, ballVecs, vecOf } = t;
  const dim = result.summary.dim;
  const perK = new Map(KS.map((k) => [k, { hit: 0, gt: 0, rec: 0 }]));
  const perDelta = new Map(DELTAS.map((d) => [d, { hit: 0, gt: 0, rec: 0 }]));
  const baselines = [];

  for (const pq of result.perQuery) {
    const qv = vecOf(pq.query);
    if (!qv) throw new Error(`${tag}: 缓存缺查询向量「${pq.query}」`);
    // 全库余弦 + 按值降序的球下标
    const sims = ballVecs.map((bv) => cos(qv, bv));
    const order = sims.map((s, i) => i).sort((x, y) => sims[y] - sims[x]);
    // 召回集：result 里记录的 id → 语料下标（球 id 从 1 开始顺序分配）
    const recalledIdx = new Set(pq.recalled.map((r) => r.id - 1));
    // 随机基线：该查询下全库余弦均值（与尺度对齐的噪声水位）
    const baseline = mean(sims);
    baselines.push(baseline);

    for (const k of KS) {
      const gt = new Set(order.slice(0, k));
      const hit = [...recalledIdx].filter((i) => gt.has(i)).length;
      const agg = perK.get(k);
      agg.hit += hit;
      agg.gt += k;
      agg.rec += recalledIdx.size;
    }
    for (const d of DELTAS) {
      const thr = baseline + d;
      let gtCount = 0;
      for (const s of sims) if (s >= thr) gtCount++;
      const hit = [...recalledIdx].filter((i) => sims[i] >= thr).length;
      const agg = perDelta.get(d);
      agg.hit += hit;
      agg.gt += gtCount;
      agg.rec += recalledIdx.size;
    }
  }
  const n = result.perQuery.length;
  return {
    tag,
    dim,
    model: result.summary.model || '—',
    meanBaseline: mean(baselines),
    perK: [...perK].map(([k, a]) => ({
      k,
      recall: a.hit / a.gt,
      precision: a.hit / a.rec,
      gtPerQuery: a.gt / n,
    })),
    perDelta: [...perDelta].map(([d, a]) => ({
      delta: d,
      recall: a.gt ? a.hit / a.gt : null,
      gtPerQuery: a.gt / n,
    })),
  };
}

const rows = TAGS.map((tag) => analyse(loadTag(tag)));

console.log('=== 模型 ===');
for (const r of rows) {
  console.log(`${r.tag.padEnd(6)} dim=${String(r.dim).padEnd(5)} 随机基线=${r.meanBaseline.toFixed(4)}`);
}

console.log('\n=== recall@K（相关集 = 全库 cos 最高的 K 个，尺度无关）===');
console.log('K'.padStart(5) + rows.map((r) => r.tag.padStart(12)).join(''));
for (const k of KS) {
  const cells = rows.map((r) => pct(r.perK.find((x) => x.k === k).recall).padStart(12));
  console.log(String(k).padStart(5) + cells.join(''));
}
console.log('\n=== precision@K（召回集中落在 Top-K 的比例）===');
console.log('K'.padStart(5) + rows.map((r) => r.tag.padStart(12)).join(''));
for (const k of KS) {
  const cells = rows.map((r) => pct(r.perK.find((x) => x.k === k).precision).padStart(12));
  console.log(String(k).padStart(5) + cells.join(''));
}

console.log('\n=== 阈值按各模型随机基线平移（相关集 = cos >= baseline + Δ）===');
console.log('Δ'.padStart(6) + rows.map((r) => r.tag.padStart(12)).join('') + '   （相关集均值）');
for (const d of DELTAS) {
  const cells = rows.map((r) => pct(r.perDelta.find((x) => x.delta === d).recall).padStart(12));
  const counts = rows.map((r) => r.perDelta.find((x) => x.delta === d).gtPerQuery.toFixed(1)).join(' / ');
  console.log(`+${d.toFixed(2)}`.padStart(6) + cells.join('') + `   ${counts}`);
}

fs.writeFileSync(
  path.join(__dirname, 'relevance-compare.json'),
  JSON.stringify(rows, null, 2),
  'utf8',
);
console.log('\n明细已写入 test/relevance-compare.json');

// ============================================================
// 配对比较：同一批查询上逐条对比（n=30，比均值更有说服力）
// ============================================================
if (rows.length === 2) {
  const [a, b] = TAGS;
  const ra = JSON.parse(fs.readFileSync(path.join(__dirname, `relevance-result-${a}.json`), 'utf8'));
  const rb = JSON.parse(fs.readFileSync(path.join(__dirname, `relevance-result-${b}.json`), 'utf8'));
  console.log(`\n=== 配对比较（${a} vs ${b}，逐查询）===`);
  for (const key of ['recallAt20', 'precisionAt20', 'meanCos']) {
    let win = 0, tie = 0, lose = 0;
    let sumA = 0, sumB = 0;
    for (let i = 0; i < ra.perQuery.length; i++) {
      const x = ra.perQuery[i][key];
      const y = rb.perQuery[i][key];
      sumA += x; sumB += y;
      if (Math.abs(x - y) < 1e-9) tie++;
      else if (x > y) win++;
      else lose++;
    }
    console.log(
      `${key.padEnd(14)} ${a} 胜 ${String(win).padStart(2)} / 平 ${String(tie).padStart(2)} / 负 ${String(lose).padStart(2)}` +
      `   均值 ${(sumA / ra.perQuery.length).toFixed(4)} vs ${(sumB / rb.perQuery.length).toFixed(4)}`,
    );
  }
  // 符号检验（双侧，正态近似）
  const signTest = (win, lose) => {
    const n = win + lose;
    if (!n) return 1;
    const z = (Math.abs(win - lose) - 1) / Math.sqrt(n);
    const p = 2 * (1 - 0.5 * (1 + erf(z / Math.SQRT2)));
    return Math.max(0, Math.min(1, p));
  };
  const erf = (x) => {
    const s = Math.sign(x);
    x = Math.abs(x);
    const a1 = 0.254829592, a2 = -0.284496736, a3 = 1.421413741, a4 = -1.453152027, a5 = 1.061405429, p = 0.3275911;
    const t = 1 / (1 + p * x);
    const y = 1 - ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * Math.exp(-x * x);
    return s * y;
  };
  let w = 0, l = 0;
  for (let i = 0; i < ra.perQuery.length; i++) {
    if (ra.perQuery[i].recallAt20 > rb.perQuery[i].recallAt20) w++;
    else if (ra.perQuery[i].recallAt20 < rb.perQuery[i].recallAt20) l++;
  }
  console.log(`recall@20 符号检验 p = ${signTest(w, l).toFixed(3)}（n=${w + l}）`);
}

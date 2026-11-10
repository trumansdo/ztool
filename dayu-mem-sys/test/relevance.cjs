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
// 运行：node test/relevance.cjs --balls 5000 --budget 10 --top 5

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const repoRoot = path.resolve(__dirname, '..');
process.chdir(repoRoot);

const api = require(path.join(__dirname, 'dayu_test.node'));
const CONFIG = (() => {
  const i = process.argv.indexOf('--config');
  return path.resolve(i >= 0 ? process.argv[i + 1] : 'test/config-http.toml');
})();
const TAG = (() => {
  const i = process.argv.indexOf('--tag');
  if (i >= 0) return process.argv[i + 1];
  // 默认按配置文件名取 tag，保证不同嵌入模型的缓存/结果互不覆盖
  return path.basename(CONFIG, '.toml').replace(/^config-/, '');
})();
const baseToml = fs.readFileSync(CONFIG, 'utf8');
const { CORPUS, QUERIES } = require(path.join(__dirname, 'corpus.cjs'));

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
}
const BALLS = +arg('balls', 5000);
const BUDGET = +arg('budget', 10);
const TOP = +arg('top', 5);
const EMBED_CHUNK = 64;

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

function buildEdges(mems) {
  const a = [];
  const b = [];
  const w = [];
  const byDomain = new Map();
  mems.forEach((m, i) => {
    if (!byDomain.has(m.domain)) byDomain.set(m.domain, []);
    byDomain.get(m.domain).push(i);
  });
  for (const members of byDomain.values()) {
    const m = members.length;
    if (m < 2) continue;
    for (let j = 0; j < m; j++) {
      for (let k = 1; k <= 2; k++) {
        const x = members[j];
        const y = members[(j + k * 7) % m];
        if (x === y) continue;
        a.push(x);
        b.push(y);
        w.push(0.5 + ((j + k) % 5) * 0.1);
      }
    }
  }
  for (let i = 0; i < mems.length; i += 40) {
    const x = i;
    const y = (i + 137) % mems.length;
    if (x === y) continue;
    a.push(x);
    b.push(y);
    w.push(0.05 + (i % 3) * 0.05);
  }
  return { a, b, w };
}

// 缓存必须按模型隔离：文本相同但模型不同，向量维度与语义空间都不同
const CACHE_PATH = path.join(__dirname, `embed-cache-${TAG}.json`);
let cache = fs.existsSync(CACHE_PATH) ? JSON.parse(fs.readFileSync(CACHE_PATH, 'utf8')) : {};
const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');

async function embedAll(texts) {
  const uniq = [...new Set(texts)];
  const missing = uniq.filter((t) => !cache[hashOf(t)]);
  for (let i = 0; i < missing.length; i += EMBED_CHUNK) {
    const part = missing.slice(i, i + EMBED_CHUNK);
    const vs = await api.embedBatch(part);
    part.forEach((t, k) => {
      cache[hashOf(t)] = vs[k];
    });
  }
  if (missing.length) fs.writeFileSync(CACHE_PATH, JSON.stringify(cache), 'utf8');
  return texts.map((t) => cache[hashOf(t)]);
}

const cos = (a, b) => {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += a[i] * b[i];
  return s;
};

const pct = (arr, p) => {
  if (!arr.length) return null;
  const s = [...arr].sort((x, y) => x - y);
  const idx = Math.min(s.length - 1, Math.max(0, Math.round((p / 100) * (s.length - 1))));
  return s[idx];
};
const mean = (arr) => (arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : 0);

(async () => {
  const tag = `rel-${TAG}-${BALLS}`;
  for (const f of fs.readdirSync('test')) {
    if (f.startsWith(`${tag}.tdb`)) fs.unlinkSync(path.join('test', f));
  }

  // 放宽注入预算：让尽可能多的点亮球都渲染出来，测的是「召回」而不是「预算截断」
  const toml = baseToml
    .replace(/^db_path\s*=.*$/m, `db_path = "test/${tag}.tdb"`)
    .replace(/^total_water_budget\s*=.*$/m, `total_water_budget = ${BUDGET}`)
    .replace(/^start_top_n\s*=.*$/m, `start_top_n = ${TOP}`)
    .replace(/^token_budget\s*=.*$/m, 'token_budget = 1000000')
    .replace(/^max_balls_per_inject\s*=.*$/m, 'max_balls_per_inject = 5000')
    .replace(/^log_level\s*=.*$/m, 'log_level = "warn"')
    .replace(/^log_dir\s*=.*$/m, 'log_dir = "test/logs-http"');

  api.init(toml);
  const dim = (await api.getStats()).embeddingDim;

  const mems = expandCorpus(BALLS);
  const vectors = await embedAll(mems.map((m) => m.text));
  const ids = [];
  for (let i = 0; i < mems.length; i += 2000) {
    const part = mems.slice(i, i + 2000);
    const got = await api.createBallsBatch(
      part.map((p) => p.text),
      vectors.slice(i, i + 2000),
      part.map(() => 1.0),
    );
    ids.push(...got);
  }
  const { a, b, w } = buildEdges(mems);
  for (let i = 0; i < a.length; i += 5000) {
    const end = Math.min(i + 5000, a.length);
    await api.linkBallsBatch(
      a.slice(i, end).map((x) => ids[x]),
      b.slice(i, end).map((x) => ids[x]),
      w.slice(i, end),
    );
  }

  const idToIdx = new Map(ids.map((id, i) => [id, i]));
  const queryVecs = await embedAll(QUERIES.map((q) => q.q));

  const perQuery = [];
  for (let qi = 0; qi < QUERIES.length; qi++) {
    const { q, domain } = QUERIES[qi];
    const qv = queryVecs[qi];

    // 全库余弦 → 真值排序
    const all = new Array(BALLS);
    for (let i = 0; i < BALLS; i++) all[i] = cos(qv, vectors[i]);
    const order = Array.from({ length: BALLS }, (_, i) => i).sort((x, y) => all[y] - all[x]);
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
    // 随机基线：抽 5 个球
    const rnd = [];
    for (let k = 0; k < 5; k++) rnd.push(all[(qi * 977 + k * 131) % BALLS]);

    perQuery.push({
      query: q,
      domain,
      gtBest: all[order[0]],
      gtTop20Floor: all[order[19]],
      litCount: r.stats.litCount,
      recalledCount: recalled.length,
      meanCos: mean(cosines),
      medianCos: pct(cosines, 50),
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
    balls: BALLS,
    dim,
    budget: BUDGET,
    topN: TOP,
    queries: perQuery.length,
    recalledTotal: allCos.length,
    meanCos: +mean(allCos).toFixed(4),
    medianCos: +pct(allCos, 50).toFixed(4),
    p10: +pct(allCos, 10).toFixed(4),
    p90: +pct(allCos, 90).toFixed(4),
    buckets: buckets.map(([label, fn]) => ({
      label,
      count: allCos.filter(fn).length,
      ratio: +(allCos.filter(fn).length / allCos.length).toFixed(3),
    })),
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
  for (const b of summary.buckets) console.log(`${b.label.padEnd(16)} ${String(b.count).padStart(5)}  ${(b.ratio * 100).toFixed(1)}%`);
  console.log('\n=== 每查询 ===');
  console.log('查询 | 点亮 | 召回 | 均值cos | 中位 | 精确率@20 | 召回率@20 | 随机基线 | 真值最佳');
  for (const p of perQuery) {
    console.log(
      `${p.query.slice(0, 18).padEnd(18)} | ${String(p.litCount).padStart(3)} | ${String(p.recalledCount).padStart(3)} | ` +
        `${p.meanCos.toFixed(3)} | ${p.medianCos.toFixed(3)} | ${p.precisionAt20.toFixed(2)} | ` +
        `${p.recallAt20.toFixed(2)} | ${p.randomBaseline.toFixed(3)} | ${p.gtBest.toFixed(3)}`,
    );
  }

  fs.writeFileSync(`test/relevance-result-${TAG}.json`, JSON.stringify({ summary, perQuery }, null, 2), 'utf8');
  console.log(`明细已写入 test/relevance-result-${TAG}.json`);
})().catch((e) => {
  console.error('FATAL', e);
  process.exit(1);
});

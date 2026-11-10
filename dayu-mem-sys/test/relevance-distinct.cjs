// 离线分析：从 relevance-result.json 与 embed-cache.json 计算
//   - 原始（含重复文本）与去重后的精确率 / 召回率
//   - 语义相关率分档
// 不调 API，不建库。

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const dir = __dirname;
const { CORPUS, QUERIES } = require(path.join(dir, 'corpus.cjs'));
const cache = JSON.parse(fs.readFileSync(path.join(dir, 'embed-cache.json'), 'utf8'));
const { summary, perQuery } = JSON.parse(fs.readFileSync(path.join(dir, 'relevance-result.json'), 'utf8'));

const DOMAINS = Object.keys(CORPUS);
const LEADINS = [
  '', '记一下：', '补充一点：', '昨天想到：', '和同事聊到：', '看文章时记的：', '复盘：',
  '再确认一下：', '突然想起来：', '周会上提过：', '整理笔记：', '踩坑记录：', '值得记：',
  '睡前回顾：', '路上想到：', '技术分享里听到：', '文档里看到：', '实践下来：', '重新整理：', '给自己提醒：',
];
const BALLS = summary.balls;

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

/** 去掉口语前缀，还原成基础句 */
function baseOf(text) {
  for (const lead of LEADINS) {
    if (lead && text.startsWith(lead)) return text.slice(lead.length);
  }
  return text;
}

const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');
const cos = (a, b) => {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += a[i] * b[i];
  return s;
};
const mean = (a) => (a.length ? a.reduce((x, y) => x + y, 0) / a.length : 0);

const mems = expandCorpus(BALLS);
const vectors = mems.map((m) => cache[hashOf(m.text)]);

// 每个基础句的最佳余弦（副本取最大）
const baseIndex = new Map(); // base text -> [ball idx]
mems.forEach((m, i) => {
  const b = baseOf(m.text);
  if (!baseIndex.has(b)) baseIndex.set(b, []);
  baseIndex.get(b).push(i);
});

const rows = [];
for (const p of perQuery) {
  const qv = cache[hashOf(p.query)];
  const all = vectors.map((v) => cos(qv, v));

  // 去重真值：每个基础句取其所有副本里的最大余弦
  const baseBest = [...baseIndex.entries()].map(([text, idxs]) => ({
    text,
    cos: Math.max(...idxs.map((i) => all[i])),
  }));
  baseBest.sort((a, b) => b.cos - a.cos);
  const gtBase20 = new Set(baseBest.slice(0, 20).map((x) => x.text));

  const recalledBases = new Set(p.recalled.map((x) => baseOf(x.text)));
  const hitBase = [...recalledBases].filter((b) => gtBase20.has(b)).length;

  rows.push({
    query: p.query,
    lit: p.litCount,
    distinctRecalled: recalledBases.size,
    precisionRaw: p.precisionAt20,
    recallRaw: p.recallAt20,
    precisionDistinct: recalledBases.size ? hitBase / recalledBases.size : null,
    recallDistinct: hitBase / 20,
    meanCos: p.meanCos,
    medianCos: p.medianCos,
    gtBest: p.gtBest,
    randomBaseline: p.randomBaseline,
  });
}

const f = (x) => (x === null ? 'n/a' : x.toFixed(3));
console.log('=== 去重前后对比 ===');
console.log('查询 | 点亮 | 去重召回 | 精确率(原始) | 精确率(去重) | 召回率(原始) | 召回率(去重)');
for (const r of rows) {
  console.log(
    `${r.query.slice(0, 16).padEnd(16)} | ${String(r.lit).padStart(3)} | ${String(r.distinctRecalled).padStart(3)} | ` +
      `${f(r.precisionRaw)} | ${f(r.precisionDistinct)} | ${f(r.recallRaw)} | ${f(r.recallDistinct)}`,
  );
}

console.log('\n=== 均值 ===');
console.log(`精确率(原始, 含重复) = ${mean(rows.map((r) => r.precisionRaw)).toFixed(3)}`);
console.log(`精确率(去重)         = ${mean(rows.map((r) => r.precisionDistinct)).toFixed(3)}`);
console.log(`召回率(原始, 含重复) = ${mean(rows.map((r) => r.recallRaw)).toFixed(3)}`);
console.log(`召回率(去重)         = ${mean(rows.map((r) => r.recallDistinct)).toFixed(3)}`);
console.log(`平均去重召回条数     = ${mean(rows.map((r) => r.distinctRecalled)).toFixed(1)}`);

const allCos = perQuery.flatMap((p) => p.recalled.map((x) => x.cos));
console.log('\n=== 语义相关率（全量召回球）===');
console.log(`样本 ${allCos.length} 个球，均值 ${mean(allCos).toFixed(3)}，中位 ${[...allCos].sort((a,b)=>a-b)[Math.floor(allCos.length/2)].toFixed(3)}`);
console.log(`随机基线 ${mean(perQuery.map((p) => p.randomBaseline)).toFixed(3)} → 提升 ${(mean(allCos) - mean(perQuery.map((p) => p.randomBaseline))).toFixed(3)}`);

// 强线索 vs 普通
const strong = perQuery.filter((p) => p.gtBest >= 0.8);
const normal = perQuery.filter((p) => p.gtBest < 0.8);
for (const [name, g] of [['强线索(best>=0.8)', strong], ['普通(best<0.8)', normal]]) {
  const c = g.flatMap((p) => p.recalled.map((x) => x.cos));
  const geq05 = c.filter((x) => x >= 0.5).length / c.length;
  const geq04 = c.filter((x) => x >= 0.4).length / c.length;
  console.log(
    `${name.padEnd(18)} 查询 ${String(g.length).padStart(2)} 个 | 召回 ${String(c.length).padStart(3)} 球 | ` +
      `均值 ${mean(c).toFixed(3)} | >=0.5 ${(geq05 * 100).toFixed(0)}% | >=0.4 ${(geq04 * 100).toFixed(0)}% | 平均点亮 ${mean(g.map((p) => p.litCount)).toFixed(1)}`,
  );
}

fs.writeFileSync(path.join(dir, 'relevance-distinct.json'), JSON.stringify(rows, null, 2), 'utf8');

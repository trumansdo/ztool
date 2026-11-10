// 相关率（召回率）：|召回 ∩ 相关| / |相关|
//
// 「相关」不再用人为的 Top-20，而是按余弦阈值定义：
//   相关集 = 全库中 cos(query, ball) >= 阈值的所有球
// 阈值取 0.4 / 0.5 / 0.6 三档（实测：同领域 0.64~0.85，跨领域 0.19~0.31，随机基线 0.37）
//
// 同时给出去重版（语料 375 基础句铺成 5000 球，每句约 13 份副本）。
//
// 运行：node test/relevance-rate.cjs

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const dir = __dirname;
const TAG = (() => {
  const i = process.argv.indexOf('--tag');
  return i >= 0 ? process.argv[i + 1] : 'http';
})();
const { CORPUS, QUERIES } = require(path.join(dir, 'corpus.cjs'));
const cache = JSON.parse(fs.readFileSync(path.join(dir, `embed-cache-${TAG}.json`), 'utf8'));
const { summary, perQuery } = JSON.parse(fs.readFileSync(path.join(dir, `relevance-result-${TAG}.json`), 'utf8'));

const DOMAINS = Object.keys(CORPUS);
const LEADINS = [
  '', '记一下：', '补充一点：', '昨天想到：', '和同事聊到：', '看文章时记的：', '复盘：',
  '再确认一下：', '突然想起来：', '周会上提过：', '整理笔记：', '踩坑记录：', '值得记：',
  '睡前回顾：', '路上想到：', '技术分享里听到：', '文档里看到：', '实践下来：', '重新整理：', '给自己提醒：',
];
const BALLS = summary.balls;
const THRESHOLDS = [0.4, 0.5, 0.6];

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

const baseOf = (text) => {
  for (const lead of LEADINS) {
    if (lead && text.startsWith(lead)) return text.slice(lead.length);
  }
  return text;
};

const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');
const cos = (a, b) => {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += a[i] * b[i];
  return s;
};
const mean = (a) => (a.length ? a.reduce((x, y) => x + y, 0) / a.length : 0);
const pct = (a, p) => {
  if (!a.length) return 0;
  const s = [...a].sort((x, y) => x - y);
  return s[Math.min(s.length - 1, Math.max(0, Math.round((p / 100) * (s.length - 1))))];
};

const mems = expandCorpus(BALLS);
const vectors = mems.map((m) => cache[hashOf(m.text)]);

// 基础句 → 副本索引
const baseIndex = new Map();
mems.forEach((m, i) => {
  const b = baseOf(m.text);
  if (!baseIndex.has(b)) baseIndex.set(b, []);
  baseIndex.get(b).push(i);
});

const rows = [];
for (const p of perQuery) {
  const qv = cache[hashOf(p.query)];
  const all = vectors.map((v) => cos(qv, v));

  // 召回集合（球 id）
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

    const hitBalls = relevant.filter((i) => recalledIds.has(i + 1)).length; // 球 id 从 1 开始
    const hitBases = [...recalledBases].filter((b) => relevantBases.has(b)).length;

    row[`relevant@${th}`] = relevant.length;
    row[`recall@${th}`] = relevant.length ? hitBalls / relevant.length : null;
    row[`relevantBase@${th}`] = relevantBases.size;
    row[`recallBase@${th}`] = relevantBases.size ? hitBases / relevantBases.size : null;
  }
  rows.push(row);
}

const f3 = (x) => (x === null ? 'n/a' : x.toFixed(3));
const f2 = (x) => (x === null ? 'n/a' : (x * 100).toFixed(1) + '%');

console.log('=== 相关率（召回 ∩ 相关 / 相关）===');
console.log('查询 | 相关@0.4 | 相关@0.5 | 相关@0.6 | 召回 | 相关率@0.4 | @0.5 | @0.6');
for (const r of rows) {
  console.log(
    `${r.query.slice(0, 16).padEnd(16)} | ${String(r['relevant@0.4']).padStart(5)} | ${String(r['relevant@0.5']).padStart(5)} | ` +
      `${String(r['relevant@0.6']).padStart(5)} | ${String(r.recalled).padStart(3)} | ` +
      `${f2(r['recall@0.4']).padStart(6)} | ${f2(r['recall@0.5']).padStart(6)} | ${f2(r['recall@0.6']).padStart(6)}`,
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
console.log('\n=== 分组 ===');
for (const [name, g] of [
  ['强线索(best>=0.8)', rows.filter((r) => r['relevant@0.6'] > 0 && r['recall@0.6'] !== null && g0(r))],
  ['普通(best<0.8)', rows.filter((r) => !g0(r))],
]) {
  console.log(
    `${name.padEnd(18)} 查询 ${String(g.length).padStart(2)} | 平均点亮 ${mean(g.map((r) => r.lit)).toFixed(1)} | ` +
      `相关球@0.4 ${mean(g.map((r) => r['relevant@0.4'])).toFixed(0)} | 召回率@0.4 ${f2(mean(g.map((r) => r['recall@0.4'])))} | ` +
      `@0.5 ${f2(mean(g.map((r) => r['recall@0.5'])))} | @0.6 ${f2(mean(g.map((r) => r['recall@0.6'])))}`,
  );
}

function g0(r) {
  const p = perQuery.find((x) => x.query === r.query);
  return p && p.gtBest >= 0.8;
}

fs.writeFileSync(path.join(dir, 'relevance-rate.json'), JSON.stringify(rows, null, 2), 'utf8');
console.log('\n明细已写入 test/relevance-rate.json');

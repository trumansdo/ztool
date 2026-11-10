// 上限测算：如果起始球做「同义去重」（Top-5 取 5 个不同基础句），相关率能到多少
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
const LEADINS = ['', '记一下：', '补充一点：', '昨天想到：', '和同事聊到：', '看文章时记的：', '复盘：', '再确认一下：', '突然想起来：', '周会上提过：', '整理笔记：', '踩坑记录：', '值得记：', '睡前回顾：', '路上想到：', '技术分享里听到：', '文档里看到：', '实践下来：', '重新整理：', '给自己提醒：'];
const BALLS = summary.balls;
function expandCorpus(n) {
  const out = [];
  const maxPerDomain = Math.max(...DOMAINS.map((d) => CORPUS[d].length));
  for (let r = 0; r < maxPerDomain && out.length < n; r++)
    for (const domain of DOMAINS) { const t = CORPUS[domain][r]; if (t === undefined) continue; out.push({ text: t, domain }); if (out.length >= n) break; }
  let i = 0; const bc = out.length;
  while (out.length < n) { const base = out[i % bc]; const lead = LEADINS[(1 + Math.floor(i / bc)) % LEADINS.length]; out.push({ text: `${lead}${base.text}`, domain: base.domain }); i++; }
  return out;
}
const baseOf = (t) => { for (const l of LEADINS) if (l && t.startsWith(l)) return t.slice(l.length); return t; };
const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');
const cos = (a, b) => { let s = 0; for (let i = 0; i < a.length; i++) s += a[i] * b[i]; return s; };
const mean = (a) => (a.length ? a.reduce((x, y) => x + y, 0) / a.length : 0);

const mems = expandCorpus(BALLS);
const vectors = mems.map((m) => cache[hashOf(m.text)]);
const baseIndex = new Map();
mems.forEach((m, i) => { const b = baseOf(m.text); if (!baseIndex.has(b)) baseIndex.set(b, []); baseIndex.get(b).push(i); });

const out = [];
for (const p of perQuery) {
  const qv = cache[hashOf(p.query)];
  const all = vectors.map((v) => cos(qv, v));
  // 每个基础句的最佳余弦
  const perBase = [...baseIndex.entries()].map(([text, idxs]) => ({ text, cos: Math.max(...idxs.map((i) => all[i])) }));
  perBase.sort((a, b) => b.cos - a.cos);
  const top5Distinct = perBase.slice(0, 5).map((x) => x.text);

  const r = {};
  for (const th of [0.4, 0.5, 0.6]) {
    const relevantBases = new Set(perBase.filter((x) => x.cos >= th).map((x) => x.text));
    const actualRecalled = new Set(p.recalled.map((x) => baseOf(x.text)));
    const hitActual = [...actualRecalled].filter((b) => relevantBases.has(b)).length;
    const hitCeiling = top5Distinct.filter((b) => relevantBases.has(b)).length;
    r[`now@${th}`] = relevantBases.size ? hitActual / relevantBases.size : null;
    r[`ceil@${th}`] = relevantBases.size ? hitCeiling / relevantBases.size : null;
    r[`rel@${th}`] = relevantBases.size;
  }
  out.push({ query: p.query, lit: p.litCount, distinctRecalled: new Set(p.recalled.map((x) => baseOf(x.text))).size, ...r });
}

const f2 = (x) => (x === null ? 'n/a' : (x * 100).toFixed(1) + '%');
console.log('查询 | 去重召回 | 相关@0.5 | 现相关率@0.5 | 去重后@0.5 | 相关@0.6 | 现相关率@0.6 | 去重后@0.6');
for (const r of out)
  console.log(`${r.query.slice(0, 16).padEnd(16)} | ${String(r.distinctRecalled).padStart(3)} | ${String(r['rel@0.5']).padStart(3)} | ${f2(r['now@0.5']).padStart(6)} | ${f2(r['ceil@0.5']).padStart(6)} | ${String(r['rel@0.6']).padStart(3)} | ${f2(r['now@0.6']).padStart(6)} | ${f2(r['ceil@0.6']).padStart(6)}`);
console.log('\n平均：');
for (const th of [0.4, 0.5, 0.6])
  console.log(`  @${th}: 现在 ${f2(mean(out.map((r) => r[`now@${th}`])))}  → 去重后上限 ${f2(mean(out.map((r) => r[`ceil@${th}`])))}`);

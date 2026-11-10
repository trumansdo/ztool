// 诊断：为什么部分查询只点亮 5 个球，另一部分点亮 25+
//
// 复用已建好的库（test/rel-5000.tdb），不重建。

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const repoRoot = path.resolve(__dirname, '..');
process.chdir(repoRoot);

const api = require(path.join(__dirname, 'dayu_test.node'));
const baseToml = fs.readFileSync(path.join(__dirname, 'config-http.toml'), 'utf8');
const { CORPUS, QUERIES } = require(path.join(__dirname, 'corpus.cjs'));

const DOMAINS = Object.keys(CORPUS);
const LEADINS = [
  '', '记一下：', '补充一点：', '昨天想到：', '和同事聊到：', '看文章时记的：', '复盘：',
  '再确认一下：', '突然想起来：', '周会上提过：', '整理笔记：', '踩坑记录：', '值得记：',
  '睡前回顾：', '路上想到：', '技术分享里听到：', '文档里看到：', '实践下来：', '重新整理：', '给自己提醒：',
];
const BALLS = 5000;

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

const CACHE_PATH = path.join(__dirname, 'embed-cache.json');
const cache = JSON.parse(fs.readFileSync(CACHE_PATH, 'utf8'));
const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');
const cos = (a, b) => {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += a[i] * b[i];
  return s;
};

const toml = baseToml
  .replace(/^db_path\s*=.*$/m, 'db_path = "test/rel-5000.tdb"')
  .replace(/^total_water_budget\s*=.*$/m, 'total_water_budget = 10')
  .replace(/^start_top_n\s*=.*$/m, 'start_top_n = 5')
  .replace(/^token_budget\s*=.*$/m, 'token_budget = 1000000')
  .replace(/^max_balls_per_inject\s*=.*$/m, 'max_balls_per_inject = 5000')
  .replace(/^log_level\s*=.*$/m, 'log_level = "warn"')
  .replace(/^log_dir\s*=.*$/m, 'log_dir = "test/logs-http"');

api.init(toml);

(async () => {
  const mems = expandCorpus(BALLS);
  const vectors = mems.map((m) => cache[hashOf(m.text)]);
  const list = await api.listBalls(null);
  const ids = list.map((b) => b.id).sort((a, b) => a - b);
  const idToIdx = new Map(ids.map((id, i) => [id, i]));

  const probes = [
    QUERIES.find((q) => q.q.includes('编译警告')),
    QUERIES.find((q) => q.q.includes('新功能没人用')),
  ];

  for (const { q } of probes) {
    const qv = cache[hashOf(q)];
    const all = vectors.map((v) => cos(qv, v));
    const order = Array.from({ length: BALLS }, (_, i) => i).sort((x, y) => all[y] - all[x]);

    const r = await api.recall(q, 1.0);
    console.log(`\n=== ${q} ===`);
    console.log(
      `起始=${r.stats.startCount} 点亮=${r.stats.litCount} 被挡=${r.stats.blockedPipeCount} 注入块=${r.blocks.length}`,
    );
    console.log(`全库真值 top8: ${order.slice(0, 8).map((i) => all[i].toFixed(3)).join(' ')}`);

    const recalled = [];
    for (const blk of r.blocks) {
      for (const bid of blk.ballIds || []) {
        const idx = idToIdx.get(bid);
        if (idx !== undefined) recalled.push({ cos: all[idx], text: mems[idx].text });
      }
    }
    recalled.sort((a, b) => b.cos - a.cos);
    console.log(`召回 ${recalled.length} 个，cos 分布：`);
    for (const x of recalled.slice(0, 6)) console.log(`  ${x.cos.toFixed(3)}  ${x.text.slice(0, 42)}`);
    if (recalled.length > 6) {
      console.log('  ...');
      for (const x of recalled.slice(-4)) console.log(`  ${x.cos.toFixed(3)}  ${x.text.slice(0, 42)}`);
    }
  }

  api.shutdown();
})().catch((e) => {
  console.error('FATAL', e);
  process.exit(1);
});

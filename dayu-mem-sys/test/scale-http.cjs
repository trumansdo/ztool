// 全链路规模冒烟：几十 / 几百 / 几千 / 几万 / 十万
//
// 语料：test/corpus.cjs（15 个领域 × 25 条真实陈述句 + 30 条自然查询）
// 嵌入：SiliconFlow HTTP 接口（Qwen/Qwen3-Embedding-4B，2560 维）
//
// 运行：
//   node test/scale-http.cjs --balls 100
//   node test/scale-http.cjs --ladder            # 50/500/5000/50000/100000
//   node test/scale-http.cjs --ladder --skip 100000
//
// 产物：test/scale-http-result.json

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const repoRoot = path.resolve(__dirname, '..');
process.chdir(repoRoot);

const api = require(path.join(__dirname, 'dayu_test.node'));
const baseToml = fs.readFileSync(path.join(__dirname, 'config-http.toml'), 'utf8');
const { CORPUS, QUERIES } = require(path.join(__dirname, 'corpus.cjs'));

// ---------- 参数 ----------
function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
}
const LADDER = process.argv.includes('--ladder');
const SKIP = +arg('skip', 0);
const BUDGETS = arg('budgets', '')
  ? arg('budgets', '')
      .split(',')
      .map((s) => +s.trim())
  : null;
const TOPS = arg('tops', '')
  ? arg('tops', '')
      .split(',')
      .map((s) => +s.trim())
  : null;
const EMBED_CHUNK = +arg('chunk', 64); // 每次 HTTP 请求送多少条
const CREATE_CHUNK = +arg('create-chunk', 2000);
const LINK_CHUNK = +arg('link-chunk', 5000);

// ---------- 语料展开 ----------
const DOMAINS = Object.keys(CORPUS);

// 自然的口语化前缀，模拟同一个人不同时间记录同一件事
const LEADINS = [
  '',
  '记一下：',
  '补充一点：',
  '昨天想到：',
  '和同事聊到：',
  '看文章时记的：',
  '复盘：',
  '再确认一下：',
  '突然想起来：',
  '周会上提过：',
  '整理笔记：',
  '踩坑记录：',
  '值得记：',
  '睡前回顾：',
  '路上想到：',
  '技术分享里听到：',
  '文档里看到：',
  '实践下来：',
  '重新整理：',
  '给自己提醒：',
];

/** 把语料展开成 `n` 条记忆（按领域轮转，保证任意规模下领域分布均匀） */
function expandCorpus(n) {
  const out = [];
  const maxPerDomain = Math.max(...DOMAINS.map((d) => CORPUS[d].length));
  // 第 r 轮：每个领域取第 r 条，保证小规模时也不会只剩一两个领域
  for (let r = 0; r < maxPerDomain && out.length < n; r++) {
    for (const domain of DOMAINS) {
      const text = CORPUS[domain][r];
      if (text === undefined) continue;
      out.push({ text, domain });
      if (out.length >= n) break;
    }
  }
  // 超出语料总量时加口语化前缀再铺开
  let i = 0;
  while (out.length < n) {
    const base = out[i % (DOMAINS.length * maxPerDomain)];
    const lead = LEADINS[(1 + Math.floor(i / 375)) % LEADINS.length];
    out.push({ text: `${lead}${base.text}`, domain: base.domain });
    i++;
  }
  return out;
}

// ---------- 边：同领域 2 条（0.5~0.9），跨领域每 40 球 1 条（0.05~0.15） ----------
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

// ---------- 嵌入缓存：同一文本只请求一次 ----------
const CACHE_PATH = path.join(__dirname, 'embed-cache.json');
let cache = {};
if (fs.existsSync(CACHE_PATH)) {
  try {
    cache = JSON.parse(fs.readFileSync(CACHE_PATH, 'utf8'));
  } catch {
    cache = {};
  }
}
const hashOf = (t) => crypto.createHash('sha1').update(t).digest('hex');

function chunk(arr, size) {
  const out = [];
  for (let i = 0; i < arr.length; i += size) out.push(arr.slice(i, i + size));
  return out;
}

async function embedAll(texts) {
  const uniq = [...new Set(texts)];
  const missing = uniq.filter((t) => !cache[hashOf(t)]);
  let apiCalls = 0;
  for (const part of chunk(missing, EMBED_CHUNK)) {
    const vectors = await api.embedBatch(part);
    part.forEach((t, i) => {
      cache[hashOf(t)] = vectors[i];
    });
    apiCalls++;
  }
  if (missing.length) {
    fs.writeFileSync(CACHE_PATH, JSON.stringify(cache), 'utf8');
  }
  return { vectors: texts.map((t) => cache[hashOf(t)]), unique: uniq.length, missing: missing.length, apiCalls };
}

// ---------- 单组 ----------
async function runOne({ balls, budget, sig, startTopN }) {
  const tag = `sc-${balls}-${budget}-${sig}`;
  for (const f of fs.readdirSync('test')) {
    if (f.startsWith(`${tag}.tdb`)) fs.unlinkSync(path.join('test', f));
  }

  const toml = baseToml
    .replace(/^db_path\s*=.*$/m, `db_path = "test/${tag}.tdb"`)
    .replace(/^total_water_budget\s*=.*$/m, `total_water_budget = ${budget}`)
    .replace(/^start_top_n\s*=.*$/m, `start_top_n = ${startTopN}`)
    .replace(/^log_level\s*=.*$/m, 'log_level = "warn"')
    .replace(/^log_dir\s*=.*$/m, 'log_dir = "test/logs-http"');

  const tInit = Date.now();
  api.init(toml);
  const initMs = Date.now() - tInit;
  const dim = (await api.getStats()).embeddingDim;

  // 1. 记忆
  const mems = expandCorpus(balls);
  const texts = mems.map((m) => m.text);

  // 2. 嵌入（带缓存）
  const tEmbed = Date.now();
  const { vectors, unique, missing, apiCalls } = await embedAll(texts);
  const embedMs = Date.now() - tEmbed;

  // 3. 批量建球
  const tCreate = Date.now();
  const ids = [];
  for (const part of chunk(
    mems.map((m, i) => ({ text: m.text, vec: vectors[i] })),
    CREATE_CHUNK,
  )) {
    const got = await api.createBallsBatch(
      part.map((p) => p.text),
      part.map((p) => p.vec),
      part.map(() => 1.0),
    );
    ids.push(...got);
  }
  const createMs = Date.now() - tCreate;

  // 4. 批量建管道
  const { a, b, w } = buildEdges(mems);
  const tLink = Date.now();
  let pipeCount = 0;
  const total = a.length;
  for (let i = 0; i < total; i += LINK_CHUNK) {
    const end = Math.min(i + LINK_CHUNK, total);
    pipeCount += await api.linkBallsBatch(
      a.slice(i, end).map((x) => ids[x]),
      b.slice(i, end).map((x) => ids[x]),
      w.slice(i, end),
    );
  }
  const linkMs = Date.now() - tLink;

  // 5. 查询 + 领域命中率
  const domainOf = new Map(mems.map((m, i) => [ids[i], m.domain]));
  const rows = [];
  for (const { q, domain } of QUERIES) {
    const t = Date.now();
    let r;
    try {
      r = await api.recall(q, sig);
    } catch (e) {
      rows.push({ query: q, domain, error: e.message, elapsedMs: Date.now() - t });
      continue;
    }
    const elapsed = Date.now() - t;
    let hit = 0;
    let counted = 0;
    for (const blk of r.blocks) {
      const doms = (blk.ballIds || []).map((id) => domainOf.get(id)).filter(Boolean);
      if (!doms.length) continue;
      counted++;
      if (doms.includes(domain)) hit++;
    }
    rows.push({
      query: q,
      domain,
      elapsedMs: elapsed,
      start: r.stats.startCount,
      lit: r.stats.litCount,
      blocked: r.stats.blockedPipeCount,
      strengthened: r.stats.strengthenedCount,
      blocks: r.stats.blockCount,
      tokens: r.stats.tokenCount,
      hitBlocks: hit,
      countedBlocks: counted,
    });
  }

  const stats = await api.getStats();
  api.shutdown();

  const dbSizeMB = +(
    fs
      .readdirSync('test')
      .filter((f) => f.startsWith(`${tag}.tdb`))
      .reduce((s, f) => s + fs.statSync(path.join('test', f)).size, 0) / 1048576
  ).toFixed(1);

  const ok = rows.filter((x) => !x.error);
  const avg = (k) => (ok.length ? ok.reduce((s, x) => s + x[k], 0) / ok.length : 0);
  const hitTotal = ok.reduce((s, x) => s + x.hitBlocks, 0);
  const cntTotal = ok.reduce((s, x) => s + x.countedBlocks, 0);

  return {
    balls,
    pipes: pipeCount,
    dim,
    budget,
    sig,
    startTopN,
    initMs,
    embedMs,
    embedUnique: unique,
    embedMissing: missing,
    embedApiCalls: apiCalls,
    createMs,
    linkMs,
    dbSizeMB,
    avgLit: +avg('lit').toFixed(2),
    avgBlocked: +avg('blocked').toFixed(1),
    avgStrengthened: +avg('strengthened').toFixed(1),
    avgBlocks: +avg('blocks').toFixed(2),
    avgTokens: +avg('tokens').toFixed(1),
    avgRecallMs: +avg('elapsedMs').toFixed(0),
    domainHitRate: cntTotal ? +(hitTotal / cntTotal).toFixed(3) : null,
    failedQueries: rows.length - ok.length,
    stats,
    rows,
  };
}

// ---------- 主流程 ----------
(async () => {
  const matrix = LADDER
    ? [50, 500, 5000, 50000, 100000]
        .filter((n) => n >= SKIP)
        .map((n) => ({
          balls: n,
          budget: +arg('budget', 2.5),
          sig: +arg('sig', 1.0),
          startTopN: +arg('top', 5),
        }))
    : BUDGETS
      ? BUDGETS.map((b) => ({
          balls: +arg('balls', 5000),
          budget: b,
          sig: +arg('sig', 1.0),
          startTopN: TOPS ? TOPS[0] : +arg('top', 5),
        }))
      : TOPS
        ? TOPS.map((t) => ({
            balls: +arg('balls', 5000),
            budget: +arg('budget', 2.5),
            sig: +arg('sig', 1.0),
            startTopN: t,
          }))
        : [
            {
              balls: +arg('balls', 100),
              budget: +arg('budget', 2.5),
              sig: +arg('sig', 1.0),
              startTopN: +arg('top', 5),
            },
          ];

  const results = [];
  for (const cfg of matrix) {
    process.stdout.write(`球数=${cfg.balls} ... `);
    try {
      const r = await runOne(cfg);
      results.push(r);
      console.log(
        `dim=${r.dim} 管=${r.pipes} 点亮=${r.avgLit} 被挡=${r.avgBlocked} 注入=${r.avgBlocks} ` +
          `命中率=${r.domainHitRate} 失败查询=${r.failedQueries} 延迟=${r.avgRecallMs}ms | 嵌入=${r.embedMs}ms(${r.embedMissing}/${r.embedUnique}) ` +
          `建球=${r.createMs}ms 连边=${r.linkMs}ms db=${r.dbSizeMB}MB`,
      );
    } catch (e) {
      console.log(`FAILED: ${e.message}`);
      results.push({ ...cfg, error: e.message });
    }
    // 每组立即落盘，避免十万级跑一半中断丢失前面的结果
    fs.writeFileSync('test/scale-http-result.json', JSON.stringify(results, null, 2), 'utf8');
  }

  console.log('\n=== 汇总 ===');
  console.log('球数 | 管道 | 点亮 | 被挡 | 强化 | 注入 | token | 延迟 | 命中率 | db');
  for (const r of results) {
    if (r.error) {
      console.log(`${r.balls} | ERROR ${r.error}`);
      continue;
    }
    console.log(
      `${String(r.balls).padStart(6)} | ${String(r.pipes).padStart(6)} | ${String(r.avgLit).padStart(4)} | ` +
        `${String(r.avgBlocked).padStart(4)} | ${String(r.avgStrengthened).padStart(4)} | ${String(r.avgBlocks).padStart(4)} | ` +
        `${String(r.avgTokens).padStart(5)} | ${String(r.avgRecallMs).padStart(5)}ms | ${String(r.domainHitRate).padStart(5)} | ${r.dbSizeMB}MB`,
    );
  }
  console.log('\n明细已写入 test/scale-http-result.json');
})().catch((e) => {
  console.error('FATAL', e);
  process.exit(1);
});

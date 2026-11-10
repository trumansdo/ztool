// 语料展开与建图规则：所有规模/相关率脚本共用同一份定义
//
// 语料本体在 test/corpus.cjs（纯数据，15 领域 × 25 条 + 30 条查询）。
// 这里只做「把语料铺成 n 个球」和「按领域连边」两件事，保证任意脚本、
// 任意规模下的球序与管道完全一致——否则不同脚本的测量结果不可比。

const { CORPUS, QUERIES } = require('../corpus.cjs');

const DOMAINS = Object.keys(CORPUS);
const MAX_PER_DOMAIN = Math.max(...DOMAINS.map((d) => CORPUS[d].length));

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

/**
 * 基础句序列：按领域轮转，保证任意规模下领域分布均匀。
 * 长度 = 15 × 25 = 375。
 */
const BASE = (() => {
  const out = [];
  for (let r = 0; r < MAX_PER_DOMAIN; r++) {
    for (const domain of DOMAINS) {
      const text = CORPUS[domain][r];
      if (text === undefined) continue;
      out.push({ text, domain });
    }
  }
  return out;
})();

/**
 * 把语料铺成 n 条记忆。
 *
 * 前缀稳定性：任意 n 的序列都是同一个无限序列的前 n 项，
 * 因此 expandCorpus(100) 是 expandCorpus(5000) 的前缀，向量缓存可复用。
 */
function expandCorpus(n) {
  const out = [];
  for (let i = 0; out.length < n; i++) {
    if (i < BASE.length) {
      out.push(BASE[i]);
    } else {
      // r = 超出基础句序列后的偏移量（从 0 开始），与旧实现逐字对齐
      const r = i - BASE.length;
      const base = BASE[r % BASE.length];
      const lead = LEADINS[(1 + Math.floor(r / BASE.length)) % LEADINS.length];
      out.push({ text: `${lead}${base.text}`, domain: base.domain });
    }
  }
  return out;
}

/** 去掉口语化前缀，还原基础句 */
function baseOf(text) {
  for (const lead of LEADINS) {
    if (lead && text.startsWith(lead)) return text.slice(lead.length);
  }
  return text;
}

/**
 * 建管道：
 *   - 同领域内每条球连 2 条（权重 0.5~0.9），模拟「谈论过同一件事」
 *   - 每 40 球连 1 条跨领域弱边（0.05~0.15），模拟「隐约的关联」
 *
 * 返回以「球下标」表示的边（调用方负责映射成球 id）。
 */
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

module.exports = { CORPUS, QUERIES, DOMAINS, LEADINS, BASE, expandCorpus, baseOf, buildEdges };

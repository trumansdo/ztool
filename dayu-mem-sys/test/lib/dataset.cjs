// 数据集加载：读落地文件、校验来源、按需取前缀
//
// 数据集是**派生产物**（不入库），由 `test/gen-dataset.cjs` 从人工语料生成。
// 落地文件里记着 `sourceHash`（语料 + 展开规则 + 生成器的 sha1）：
// 改了其中任一文件而忘了重跑生成器，这里会直接报错，而不是拿旧数据继续测。
//
// 拓扑（管道）**不落地**，见文件末尾「为什么边不落地」。

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const paths = require('./paths.cjs');
const { buildEdges } = require('./corpus.cjs');

const DATASET_DIR = path.join(paths.TEST_DIR, 'datasets');

/// 影响数据集产物的三个文件：语料、展开规则、生成器本身
const SOURCE_FILES = [
  path.join(paths.TEST_DIR, 'corpus.cjs'),
  path.join(paths.TEST_DIR, 'lib', 'corpus.cjs'),
  path.join(paths.TEST_DIR, 'gen-dataset.cjs'),
];

/** 把源文件的字节按固定顺序拼起来算 sha1 —— 任一文件改动都会变 */
function sourceHash() {
  const h = crypto.createHash('sha1');
  for (const f of SOURCE_FILES) {
    h.update(paths.rel(f));
    h.update('\0');
    h.update(fs.readFileSync(f));
    h.update('\0');
  }
  return h.digest('hex');
}

/**
 * 读落地数据集并校验来源
 *
 * 不做「文件不存在就自动生成」：静默生成会让「改完语料忘了重跑」变成
 * 「测试用着新数据但你以为是旧的」——两种错误都比直接报错难查。
 */
function load(id = 'main') {
  const p = path.join(DATASET_DIR, `${id}.json`);

  if (!fs.existsSync(p)) {
    throw new Error(
      `数据集不存在：${paths.rel(p)}\n` +
        `数据集是派生产物，不入库。先生成：\n` +
        `  node test/gen-dataset.cjs ${id}`,
    );
  }

  let ds;
  try {
    ds = JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch (e) {
    throw new Error(`数据集损坏（${paths.rel(p)}）：${e.message}`);
  }

  const expected = sourceHash();
  if (ds.sourceHash !== expected) {
    throw new Error(
      `数据集已过期：\n` +
        `  文件  sourceHash=${ds.sourceHash.slice(0, 8)}（id=${ds.id}，记忆 ${ds.counts?.memories} 条）\n` +
        `  当前  sourceHash=${expected.slice(0, 8)}（corpus.cjs / lib/corpus.cjs / gen-dataset.cjs 有改动）\n` +
        `请重新生成：node test/gen-dataset.cjs ${ds.id} ${ds.generator?.params?.maxMemories ?? ''}\n` +
        `（向量缓存键含 sourceHash，会自动换代，旧文件可删）`,
    );
  }

  return ds;
}

/**
 * 取前 n 条记忆
 *
 * 前缀性质：`memories(n)` 永远是 `memories(m)` 的前 n 项（n < m），
 * 所以 `--balls 100` 与 `--balls 5000` 共用同一份向量缓存。
 */
function memories(ds, n) {
  if (!Number.isInteger(n) || n <= 0) {
    throw new Error(`记忆条数非法：${n}`);
  }
  if (n > ds.memories.length) {
    throw new Error(
      `数据集 ${ds.id} 只有 ${ds.memories.length} 条记忆，取不到 ${n} 条。\n` +
        `重新生成更大的数据集：node test/gen-dataset.cjs ${ds.id} ${n}`,
    );
  }
  return ds.memories.slice(0, n);
}

/** 按前缀建拓扑（边不落地，见文末说明） */
function edges(ds, n) {
  return buildEdges(memories(ds, n));
}

/** 数据集里所有需要嵌入的文本：记忆 + 查询（顺序固定） */
function texts(ds, n) {
  return [...memories(ds, n).map((m) => m.text), ...ds.queries.map((q) => q.q)];
}

module.exports = {
  DATASET_DIR,
  sourceHash,
  load,
  memories,
  edges,
  texts,
  queries: (ds) => ds.queries,
  domains: (ds) => ds.domains,
};

// ============================================================
// 为什么边不落地
// ============================================================
//
// 最初打算把 edges 一起写进数据集（「拓扑也是数据集语义」）。实测后放弃：
//
// `buildEdges(mems)` 的连接规则是「同领域内连到 (j + k*7) % m」，m 是**该领域
// 在当前规模下的成员数**。所以 n=100 时的图不是 n=100000 时的子图——模数变了，
// 邻居全变。要落地一份最大规模的边再按前缀过滤，小规模会退化成几乎没有边
// （每个领域 6 个成员，连出去的边全指向 6 之外）。
//
// 所以边按前缀现算。它仍然是确定性的：`buildEdges` 在 `lib/corpus.cjs` 里，
// 已计入 `sourceHash`——改规则就会触发「数据集已过期」，不会悄悄换拓扑。

// 向量缓存（JSONL，append-only）
//
// 格式：
//   第 1 行  {"_meta": {backend, model, dim, dataset, sourceHash, preprocess, normalize, createdAt}}
//   第 2 行~ {"h": "<sha1(text)>", "v": [float…]}
//
// 为什么不是一个大 JSON：
//   旧实现每批（chunk=64）都把整个对象 `JSON.stringify` 重写一遍。7554 条缓存
//   要写 ~28 GB，10 万条要写 ~3.9 TB。append-only 只写新增的那部分。
//
// 为什么第 1 行必须是 meta：
//   缓存是「同一文本在同一嵌入配置下的向量」的断言。没有这条断言，换个模型
//   但沿用同名文件就会读到错的向量——而这种情况不会报错，只会让测量结果静默失真。
//
// 这个文件只被 test/lib/harness.cjs 调用。生产代码（crates/**）不读任何缓存文件。

const fs = require('fs');
const path = require('path');

const META_KEYS = ['backend', 'model', 'dim', 'dataset', 'sourceHash', 'preprocess', 'normalize'];

/** 读缓存：返回 { meta, vectors: Map<hash, vector>, path } */
function read(p) {
  if (!fs.existsSync(p)) {
    return { meta: null, vectors: new Map(), path: p };
  }

  const lines = fs.readFileSync(p, 'utf8').split('\n');
  const vectors = new Map();
  let meta = null;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (!line) continue;

    let row;
    try {
      row = JSON.parse(line);
    } catch (e) {
      throw new Error(`向量缓存损坏（${p} 第 ${i + 1} 行）：${e.message}`);
    }

    if (row._meta) {
      if (i !== 0) throw new Error(`向量缓存损坏（${p}）：_meta 必须且只能在第 1 行`);
      meta = row._meta;
    } else if (typeof row.h === 'string' && Array.isArray(row.v)) {
      vectors.set(row.h, row.v);
    } else {
      throw new Error(`向量缓存损坏（${p} 第 ${i + 1} 行）：既不是 _meta 也不是 {h,v}`);
    }
  }

  return { meta, vectors, path: p };
}

/**
 * 打开缓存并校验身份
 *
 * 不一致直接抛错，不静默重算：静默重算会花几十分钟（或几十美元），
 * 而且等于在你不知道的情况下换了测量条件。
 */
function open(p, expected) {
  const store = read(p);

  if (store.meta === null) {
    // 新文件：写 meta 头
    fs.mkdirSync(path.dirname(p), { recursive: true });
    const meta = { ...expected, createdAt: new Date().toISOString() };
    fs.writeFileSync(p, JSON.stringify({ _meta: meta }) + '\n', 'utf8');
    store.meta = meta;
    return store;
  }

  const diff = META_KEYS.filter((k) => String(store.meta[k]) !== String(expected[k]));
  if (diff.length) {
    const fmt = (m) =>
      META_KEYS.map((k) => `${k}=${m?.[k] ?? '(缺失)'}`).join(' / ');
    throw new Error(
      `向量缓存身份不匹配：${path.relative(paths_root(), p)}\n` +
        `  缓存  ${fmt(store.meta)}  （建于 ${store.meta.createdAt ?? '未知'}）\n` +
        `  当前  ${fmt(expected)}\n` +
        `不一致的字段：${diff.join(', ')}\n` +
        `换 profile 名，或删掉这个缓存文件重算。`,
    );
  }

  return store;
}

/** 追加一批向量（一次 write，不再全量重写） */
function append(p, entries) {
  if (!entries.length) return;
  const body = entries.map(([h, v]) => JSON.stringify({ h, v })).join('\n') + '\n';
  fs.appendFileSync(p, body, 'utf8');
}

/** 供离线分析脚本（relevance-rate / relevance-compare）直接读 */
function load(p) {
  return read(p).vectors;
}

function paths_root() {
  // 延迟 require：避免 paths.cjs 与本模块互相引用时的初始化顺序问题
  return require('./paths.cjs').REPO_ROOT;
}

module.exports = { META_KEYS, read, open, append, load };

// 测试骨架：napi 加载、配置初始化、向量缓存、批量建图、元数据落盘
//
// 约定：
//   - 所有脚本必须通过 harness.init() 初始化，禁止直接调 api.init()，
//     否则 db_path / log_dir 的路径约定会被绕过
//   - 所有脚本必须通过 harness.embed() 取向量，禁止各自实现缓存

const fs = require('fs');
const path = require('path');

const paths = require('./paths.cjs');
const config = require('./config.cjs');
const cache = require('./cache.cjs');
const { hashOf } = require('./stats.cjs');

const NAPI_PATH = path.join(paths.TEST_DIR, 'dayu_test.node');

// ---------- 运行环境 ----------

/** ort 的 load-dynamic 在首次建 Session 时读这个变量 */
function ensureOrtLib() {
  if (process.env.ORT_DYLIB_PATH) return;
  const dll = path.join(paths.REPO_ROOT, 'models/onnxruntime/lib/onnxruntime.dll');
  if (fs.existsSync(dll)) process.env.ORT_DYLIB_PATH = dll;
}

let cachedApi = null;

/** 加载 napi 模块（cwd 切到仓库根，因为 db_path 是相对仓库根的） */
function api() {
  if (cachedApi) return cachedApi;
  ensureOrtLib();
  if (!fs.existsSync(NAPI_PATH)) {
    throw new Error(
      `缺少 napi 产物：${NAPI_PATH}\n` +
        `构建：cargo build -p dayu-napi --release --no-default-features --features http,onnx\n` +
        `拷贝：cp target/release/dayu_napi.dll test/dayu_test.node`,
    );
  }
  process.chdir(paths.REPO_ROOT);
  cachedApi = require(NAPI_PATH);
  return cachedApi;
}

// ---------- 初始化 ----------

/**
 * 初始化一次运行。
 *
 * @param {object} opts
 * @param {string} opts.profile       配置名（onnx / http / lms）
 * @param {object} opts.overrides     参数覆盖（total_water_budget 等）
 * @param {object} opts.run           paths.newRun() 的返回值；缺省时用长期库
 * @param {boolean} opts.reuseDb      true = 不清空库（长期库、离线复算）
 * @returns {Promise<{api, dim, toml}>}
 */
async function init({ profile, overrides = {}, run = null, reuseDb = false } = {}) {
  const a = api();

  const dbPath = run ? run.dbPath : paths.MAIN_DB;
  const logDir = run ? run.logDir : path.join(paths.DATA_DIR, 'logs');

  if (!reuseDb) paths.clearDb(dbPath);

  const toml = config.load(profile, {
    db_path: paths.rel(dbPath),
    log_dir: paths.rel(logDir),
    log_level: 'warn',
    ...overrides,
  });

  a.init(toml);
  const stats = await a.getStats();

  // 缓存身份：决定「同一文本是否应该命中同一份向量缓存」
  const identity = cacheIdentity({ profile, toml, dim: stats.embeddingDim });

  return { api: a, dim: stats.embeddingDim, toml, identity, dbPath, logDir };
}

/**
 * 从配置推导缓存身份
 *
 * backend / model 都取自 toml 的实际值而不是 profile 名：profile 只是个文件名，
 * 把 http.toml 里的模型换掉、名字不变，缓存也必须换代。
 */
function cacheIdentity({ profile, toml, dim }) {
  const kv = config.keyValues(toml);
  const backend = kv.embedding_backend ?? profile;
  const raw =
    kv.embedding_api_model ??
    kv.embedding_model_path ??
    kv.embedding_model ??
    profile;
  return { profile, backend, model: modelLabel(raw), dim };
}

/**
 * 模型标识：把文件路径压成「模型名」
 *
 * `models/infinilabs/pizza-embed-zh-small-v1/onnx/model.onnx` → `pizza-embed-zh-small-v1`
 *
 * 直接用完整路径当缓存名会得到一长串目录，既难读也容易在不同机器上不一致
 * （绝对路径随环境变），但模型名必须稳定——否则换台机器缓存就命不中了。
 */
function modelLabel(raw) {
  const s = String(raw).replace(/\\/g, '/').replace(/\/+$/, '');
  const parts = s.split('/');
  const file = parts[parts.length - 1];

  if (/^model\.(onnx|bin|gguf)$/i.test(file) && parts.length >= 2) {
    const dir = parts[parts.length - 2];
    if (dir.toLowerCase() === 'onnx' && parts.length >= 3) return parts[parts.length - 3];
    return dir;
  }
  if (/\.(onnx|bin|gguf)$/i.test(file)) return file.replace(/\.[^\.]+$/, '');
  return s;
}

// ---------- 向量缓存 ----------

/**
 * 取一批文本的向量，带磁盘缓存
 *
 * 缓存身份 = `identity`（后端 + 模型 + 维度）× `dataset`（id + sourceHash）。
 * 任一项不符直接抛错——不静默重算，也不静默沿用。
 *
 * @param texts    待嵌入文本
 * @param identity `harness.init()` 返回的 identity
 * @param dataset  `lib/dataset.cjs` 的 load() 结果，或 `adhocDataset()`
 * @param chunk    每批条数（控制单次 API 请求体大小）
 */
async function embed(texts, { identity, dataset, chunk = 64 } = {}) {
  if (!identity) throw new Error('embed() 需要 identity（harness.init() 的返回值）');
  if (!dataset) throw new Error('embed() 需要 dataset（lib/dataset.cjs 的 load() 结果）');

  const a = api();
  const cachePath = paths.cachePath(identity, dataset);
  const store = cache.open(cachePath, {
    backend: identity.backend,
    model: identity.model,
    dim: identity.dim,
    dataset: dataset.id,
    sourceHash: dataset.sourceHash,
    // 这两项写进头记录并参与比对：以后给 query 加 instruction 前缀或关掉
    // L2 归一化时，旧缓存会自动失效，而不是被当成同一种向量继续用
    preprocess: 'none',
    normalize: 'l2',
  });

  const uniq = [...new Set(texts)];
  const missing = uniq.filter((t) => !store.vectors.has(hashOf(t)));

  for (let i = 0; i < missing.length; i += chunk) {
    const part = missing.slice(i, i + chunk);
    const vectors = await a.embedBatch(part);
    const entries = part.map((t, k) => {
      store.vectors.set(hashOf(t), vectors[k]);
      return [hashOf(t), vectors[k]];
    });
    // 追加写：十万级语料中途失败也只丢最后一批，不必全部重算
    cache.append(cachePath, entries);
  }

  return {
    vectors: texts.map((t) => store.vectors.get(hashOf(t))),
    unique: uniq.length,
    missing: missing.length,
    apiCalls: Math.ceil(missing.length / chunk),
    cachePath: paths.rel(cachePath),
  };
}

/**
 * 临时文本集的缓存身份
 *
 * 用于不来自数据集的文本（比如 dedupe-threshold 的句对）。sourceHash 直接由
 * 文本内容算出：改了文本就换缓存，不改就复用。
 */
function adhocDataset(id, texts) {
  const crypto = require('crypto');
  const h = crypto.createHash('sha1');
  for (const t of texts) {
    h.update(t);
    h.update('\0');
  }
  return { id, sourceHash: h.digest('hex') };
}

// ---------- 建图 ----------

/** 批量建球，返回球 id 数组（顺序与入参一致） */
async function seedBalls(api, texts, vectors, sizes, chunk = 2000) {
  const ids = [];
  for (let i = 0; i < texts.length; i += chunk) {
    const got = await api.createBallsBatch(
      texts.slice(i, i + chunk),
      vectors.slice(i, i + chunk),
      sizes.slice(i, i + chunk),
    );
    ids.push(...got);
  }
  return ids;
}

/** 批量建管道（入参为球下标，内部映射成球 id） */
async function seedEdges(api, ids, edges, chunk = 5000) {
  const { a, b, w } = edges;
  let count = 0;
  for (let i = 0; i < a.length; i += chunk) {
    const end = Math.min(i + chunk, a.length);
    count += await api.linkBallsBatch(
      a.slice(i, end).map((x) => ids[x]),
      b.slice(i, end).map((x) => ids[x]),
      w.slice(i, end),
    );
  }
  return count;
}

/** 用同一份语料建好整张图（球 + 管道） */
async function seedGraph(api, mems, vectors, edges) {
  const ids = await seedBalls(
    api,
    mems.map((m) => m.text),
    vectors,
    mems.map(() => 1.0),
  );
  const pipes = await seedEdges(api, ids, edges);
  return { ids, pipes };
}

// ---------- 元数据 ----------

/** 把一次运行的上下文写到 meta.json（结果可复现的前提） */
function writeMeta(run, meta) {
  const git = paths.gitCommit();
  paths.writeJson(run.metaPath, {
    script: meta.script,
    tag: meta.tag ?? null,
    profile: meta.profile ?? null,
    startedAt: meta.startedAt,
    elapsedMs: meta.elapsedMs ?? null,
    git,
    params: meta.params ?? {},
    dim: meta.dim ?? null,
    db: paths.rel(run.dbPath),
    cache: meta.cache ?? null,
    // 断言结果：bin/ 下的脚本必须落盘，这样「这次跑过没过」不用翻终端输出
    checks: meta.checks ?? null,
    checks_ok: meta.checks_ok ?? null,
    note: meta.note ?? null,
  });
}

/** 读取一次运行的 meta（离线分析用） */
function readMeta(runDir) {
  const meta = paths.readJson(path.join(runDir, 'meta.json'));
  if (!meta) throw new Error(`${runDir} 不是有效的运行目录（缺 meta.json）`);
  return meta;
}

module.exports = {
  api,
  init,
  embed,
  seedBalls,
  seedEdges,
  seedGraph,
  writeMeta,
  cacheIdentity,
  adhocDataset,
  readMeta,
  NAPI_PATH,
};

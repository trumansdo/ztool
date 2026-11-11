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

  return { api: a, dim: stats.embeddingDim, toml, dbPath, logDir };
}

// ---------- 向量缓存 ----------

/**
 * 取一批文本的向量，带磁盘缓存（按嵌入模型 tag 隔离）。
 * 缓存是长期资产：重算远端嵌入要花钱、算本地模型要时间。
 */
async function embed(texts, { tag, chunk = 64 } = {}) {
  if (!tag) throw new Error('embed() 需要 tag（嵌入模型标识，用于隔离缓存）');
  const a = api();
  const cachePath = paths.cachePath(tag);
  const cache = paths.readJson(cachePath) || {};

  const uniq = [...new Set(texts)];
  const missing = uniq.filter((t) => !cache[hashOf(t)]);

  for (let i = 0; i < missing.length; i += chunk) {
    const part = missing.slice(i, i + chunk);
    const vectors = await a.embedBatch(part);
    part.forEach((t, k) => {
      cache[hashOf(t)] = vectors[k];
    });
    // 分批落盘：十万级语料中途失败不至于全部重算
    paths.writeJson(cachePath, cache);
  }

  return {
    vectors: texts.map((t) => cache[hashOf(t)]),
    unique: uniq.length,
    missing: missing.length,
    apiCalls: Math.ceil(missing.length / chunk),
    cachePath: paths.rel(cachePath),
  };
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
  readMeta,
  NAPI_PATH,
};

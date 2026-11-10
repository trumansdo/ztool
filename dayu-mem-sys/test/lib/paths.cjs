// 路径约定：所有产物位置的唯一来源
//
//   test/data/        长期资产（跨运行存活，不自动清理）
//   test/out/         一次性产物（整目录可删，删了不损失信息）
//   test/config/      配置（base.toml + 各嵌入后端只写差异）
//   test/lib/         公共逻辑
//   test/bin/         可执行脚本
//
// 所有路径对外返回相对仓库根的 POSIX 路径，因为 napi 侧会 process.chdir(repoRoot)，
// db_path / log_dir 必须相对仓库根解析。

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const TEST_DIR = path.resolve(__dirname, '..');
const REPO_ROOT = path.resolve(TEST_DIR, '..');
const DATA_DIR = path.join(TEST_DIR, 'data');
const OUT_DIR = path.join(TEST_DIR, 'out');
const CACHE_DIR = path.join(DATA_DIR, 'cache');
const CONFIG_DIR = path.join(TEST_DIR, 'config');

/** 绝对路径 → 相对仓库根的 POSIX 路径（napi 侧 cwd = REPO_ROOT） */
const rel = (abs) => path.relative(REPO_ROOT, abs).split(path.sep).join('/');

/** 本地时间戳 YYYYMMDD-HHMM */
function stamp(d = new Date()) {
  const p = (n) => String(n).padStart(2, '0');
  return (
    `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}` +
    `-${p(d.getHours())}${p(d.getMinutes())}`
  );
}

/** 向量缓存路径：按嵌入模型隔离（文本相同但模型不同，向量空间不同） */
const cachePath = (tag) => path.join(CACHE_DIR, `${tag}.json`);

/** 长期全链路库 */
const MAIN_DB = path.join(DATA_DIR, 'main.tdb');
const MAIN_HISTORY = path.join(DATA_DIR, 'main.history.jsonl');

/**
 * 新建一次运行的产物目录：out/<时间戳>-<脚本>[-<标签>]/
 *   meta.json   配置快照 + git commit + 参数 + 耗时
 *   db/         该次运行的库
 *   logs/       该次运行的日志与指标
 *   result.json 结果（由脚本自行写入）
 */
function newRun(script, tag) {
  const name = tag ? `${stamp()}-${script}-${tag}` : `${stamp()}-${script}`;
  const dir = path.join(OUT_DIR, name);
  const dbDir = path.join(dir, 'db');
  const logDir = path.join(dir, 'logs');
  fs.mkdirSync(dbDir, { recursive: true });
  fs.mkdirSync(logDir, { recursive: true });
  return {
    name,
    dir,
    dbDir,
    logDir,
    dbPath: path.join(dbDir, 'main.tdb'),
    metaPath: path.join(dir, 'meta.json'),
    resultPath: path.join(dir, 'result.json'),
  };
}

/** 列出所有运行目录（新 → 旧） */
function listRuns() {
  if (!fs.existsSync(OUT_DIR)) return [];
  return fs
    .readdirSync(OUT_DIR)
    .map((name) => path.join(OUT_DIR, name))
    .filter((dir) => fs.existsSync(path.join(dir, 'meta.json')))
    .map((dir) => ({
      dir,
      name: path.basename(dir),
      meta: readJson(path.join(dir, 'meta.json')),
      resultPath: path.join(dir, 'result.json'),
    }))
    .sort((a, b) => b.name.localeCompare(a.name));
}

function readJson(p) {
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch (e) {
    if (e.code === 'ENOENT') return null;
    throw new Error(`读取 ${p} 失败：${e.message}`);
  }
}

function writeJson(p, value) {
  fs.writeFileSync(p, JSON.stringify(value, null, 2), 'utf8');
}

/** 当前 git 提交（用于把结果钉到代码版本上） */
function gitCommit() {
  try {
    const head = execSync('git rev-parse --short HEAD', { cwd: REPO_ROOT, encoding: 'utf8' }).trim();
    const dirty = execSync('git status --porcelain', { cwd: REPO_ROOT, encoding: 'utf8' }).trim() !== '';
    return { commit: head, dirty };
  } catch {
    return { commit: 'unknown', dirty: null };
  }
}

/** 清空某个库的全部分片文件 */
function clearDb(dbPath) {
  const dir = path.dirname(dbPath);
  const base = path.basename(dbPath);
  if (!fs.existsSync(dir)) return 0;
  const hits = fs.readdirSync(dir).filter((f) => f === base || f.startsWith(`${base}.`));
  for (const f of hits) fs.unlinkSync(path.join(dir, f));
  return hits.length;
}

/** 某个库当前占用的全部字节数 */
function dbBytes(dbPath) {
  const dir = path.dirname(dbPath);
  const base = path.basename(dbPath);
  if (!fs.existsSync(dir)) return 0;
  return fs
    .readdirSync(dir)
    .filter((f) => f === base || f.startsWith(`${base}.`))
    .reduce((s, f) => s + fs.statSync(path.join(dir, f)).size, 0);
}

module.exports = {
  TEST_DIR,
  REPO_ROOT,
  DATA_DIR,
  OUT_DIR,
  CACHE_DIR,
  CONFIG_DIR,
  rel,
  stamp,
  cachePath,
  MAIN_DB,
  MAIN_HISTORY,
  newRun,
  listRuns,
  readJson,
  writeJson,
  gitCommit,
  clearDb,
  dbBytes,
};

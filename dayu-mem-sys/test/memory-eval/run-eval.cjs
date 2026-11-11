#!/usr/bin/env node
// 公开基准评测（LoCoMo / LongMemEval）一次性运行器
//
// 一次运行 = `test/memory-eval/<时间戳>-<adapter>-<dataset>[-tag]/` 一个自描述目录：
//
//   config.toml     本次运行实际使用的完整配置（单文件，服务只读它）
//   serve.log       dayu-serve 的 stdout + stderr
//   store/          各 namespace 的 TriviumDB 文件
//   result.json     mceval 产出的逐题结果（文件名由 mceval 决定，见 meta.json）
//   meta.json       参数、git 提交、耗时、产物清单
//
// 产物全部落在 test/memory-eval/ 下，与 dayu-mem-sys 自身的测试产物（test/out/）隔离。
//
// 用法：
//   node test/memory-eval/run-eval.cjs --dataset locomo --sample 100 --stratified
//   node test/memory-eval/run-eval.cjs --dataset longmemeval --split oracle --sample 50 --seed 0
//   node test/memory-eval/run-eval.cjs --adapter bm25 --dataset locomo --sample 100 --stratified
//
// 选项：
//   --adapter <name>     mceval adapter，默认 dayu（bm25 / dense / hybrid-rrf 等基线不需要起服务）
//   --config <name>      嵌入后端配置名（test/config/<name>.toml），默认 lms
//   --dataset locomo|longmemeval
//   --split oracle|s|m   仅 longmemeval
//   --sample N           题目数（必须配 --seed 或 --stratified，否则取样有偏）
//   --seed N / --stratified
//   --workers N          并行题目数，默认 4
//   --top-k N            把 start_top_n / max_balls_per_inject 调到 N（默认 10）
//   --link-weight 0.5    同会话相邻回合管道粗细；off = 纯余弦对照
//   --tag <str>          目录后缀，便于区分对照实验
//   --bin <path>         dayu-serve 可执行文件
//   --eval-repo <path>   memory-core-eval 仓库根（默认 MCEVAL_REPO 或 D:/code_space/other/memory-core-eval）

const fs = require('fs');
const path = require('path');
const { spawn, spawnSync } = require('child_process');

const paths = require('../lib/paths.cjs');
const configLib = require('../lib/config.cjs');

const EVAL_ROOT = path.join(paths.TEST_DIR, 'memory-eval');
const DEFAULT_EVAL_REPO =
  process.env.MCEVAL_REPO || 'D:/code_space/other/memory-core-eval';

// ---------- 参数 ----------

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (!a.startsWith('--')) continue;
    const key = a.slice(2);
    const next = argv[i + 1];
    if (next === undefined || next.startsWith('--')) {
      out[key] = true;
    } else {
      out[key] = next;
      i += 1;
    }
  }
  return out;
}

const args = parseArgs(process.argv.slice(2));

const ADAPTER = args.adapter || 'dayu';
const DATASET = args.dataset;
const CONFIG_NAME = args.config || 'lms';
const WORKERS = Number(args.workers || 4);
const TOP_K = Number(args['top-k'] || 10);
const LINK_WEIGHT = String(args['link-weight'] ?? '0.5');
const EVAL_REPO = path.resolve(args['eval-repo'] || DEFAULT_EVAL_REPO);

if (!DATASET || !['locomo', 'longmemeval'].includes(DATASET)) {
  console.error('用法：--dataset locomo|longmemeval [其他选项]');
  process.exit(2);
}
if (args.sample && !args.seed && !args.stratified) {
  console.error('--sample 必须配 --seed 或 --stratified：数据集按问题类型聚集，裸切片有偏');
  process.exit(2);
}
if (!fs.existsSync(path.join(EVAL_REPO, 'mceval'))) {
  console.error(`找不到 memory-core-eval：${EVAL_REPO}（用 --eval-repo 或 MCEVAL_REPO 指定）`);
  process.exit(2);
}

// ---------- 运行目录 ----------

const tagParts = [ADAPTER, DATASET];
if (args.split) tagParts.push(args.split);
if (args.tag) tagParts.push(args.tag);
const runName = `${paths.stamp()}-${tagParts.join('-')}`;
const runDir = path.join(EVAL_ROOT, runName);
const storeDir = path.join(runDir, 'store');
const serveLog = path.join(runDir, 'serve.log');
const configPath = path.join(runDir, 'config.toml');
const metaPath = path.join(runDir, 'meta.json');

// 只有需要 dayu-serve 的 adapter 才产生配置快照与库目录；
// 基线（bm25/dense）纯进程内，不应留下无意义的空 store/
const NEEDS_SERVER = ADAPTER === 'dayu';

if (NEEDS_SERVER) {
  fs.mkdirSync(storeDir, { recursive: true });
  // 配置快照：服务只读这一个文件，保证结果可复现
  fs.writeFileSync(
    configPath,
    configLib.load(CONFIG_NAME, {
      metrics_enabled: false,
      log_dir: paths.rel(path.join(runDir, 'logs')),
      start_top_n: TOP_K,
      max_balls_per_inject: TOP_K,
    }),
    'utf8',
  );
} else {
  fs.mkdirSync(runDir, { recursive: true });
}

// ---------- dayu-serve 生命周期 ----------

function findBinary(explicit) {
  if (explicit) {
    if (!fs.existsSync(explicit)) throw new Error(`dayu-serve 不存在：${explicit}`);
    return explicit;
  }
  const candidates = ['target/release/dayu-serve', 'target/debug/dayu-serve'].map((rel) =>
    path.join(paths.REPO_ROOT, rel + (process.platform === 'win32' ? '.exe' : '')),
  );
  const hit = candidates.find((p) => fs.existsSync(p));
  if (!hit) {
    throw new Error(`找不到 dayu-serve，请先构建：cargo build -p dayu-serve --release`);
  }
  return hit;
}

/** 启动服务，读 stdout 的 `LISTENING <port>` 拿端口 */
function startServer() {
  const bin = findBinary(args.bin);
  const child = spawn(
    bin,
    [
      '--config',
      configPath,
      '--store-dir',
      storeDir,
      '--link-weight',
      LINK_WEIGHT,
      '--listen',
      '127.0.0.1:0',
    ],
    { cwd: paths.REPO_ROOT, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, RUST_LOG: 'info' } },
  );

  const log = fs.createWriteStream(serveLog, { flags: 'a' });
  child.stderr.pipe(log);
  child.stdout.pipe(log, { end: false });

  return new Promise((resolve, reject) => {
    let buf = '';
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error(`dayu-serve 启动超时（60s），见 ${serveLog}`));
    }, 60_000);

    const onData = (chunk) => {
      buf += chunk.toString();
      const m = buf.match(/LISTENING (\d+)/);
      if (!m) return;
      clearTimeout(timer);
      child.stdout.off('data', onData);
      resolve({
        port: Number(m[1]),
        url: `http://127.0.0.1:${m[1]}`,
        stop: () =>
          new Promise((res) => {
            if (child.exitCode !== null) return res();
            child.once('exit', () => {
              log.end();
              res();
            });
            child.kill();
          }),
      });
    };

    child.stdout.on('data', onData);
    child.on('error', reject);
    child.on('exit', (code) => {
      if (/LISTENING/.test(buf)) return;
      clearTimeout(timer);
      reject(new Error(`dayu-serve 退出（code=${code}），见 ${serveLog}`));
    });
  });
}

// ---------- mceval ----------

function runMceval(env) {
  const argv = [
    '-m',
    'mceval.cli',
    'run',
    '--adapter',
    ADAPTER,
    '--dataset',
    DATASET,
    '--workers',
    String(WORKERS),
    '--out-dir',
    runDir,
  ];
  if (args.split) argv.push('--split', String(args.split));
  if (args.sample) argv.push('--sample', String(args.sample));
  if (args.seed) argv.push('--seed', String(args.seed));
  if (args.stratified) argv.push('--stratified');

  console.log(`[eval] ${runName}`);
  console.log(`[eval] python ${argv.join(' ')}`);

  return new Promise((resolve, reject) => {
    const child = spawn(process.env.PYTHON || 'python', argv, {
      cwd: EVAL_REPO,
      stdio: 'inherit',
      env: { ...process.env, ...env },
    });
    child.on('error', reject);
    child.on('exit', (code) =>
      code === 0 ? resolve() : reject(new Error(`mceval 退出（code=${code}）`)),
    );
  });
}

// ---------- 主流程 ----------

(async () => {
  const startedAt = Date.now();
  let server = null;
  let error = null;

  try {
    if (NEEDS_SERVER) {
      server = await startServer();
      console.log(`[eval] dayu-serve ${server.url}  link_weight=${LINK_WEIGHT}`);
    }
    await runMceval(NEEDS_SERVER ? { DAYU_SERVE_URL: server.url } : {});
  } catch (e) {
    error = String(e.message || e);
    console.error(`[eval] 失败：${error}`);
  } finally {
    if (server) await server.stop();
  }

  // mceval 的结果文件名含时间戳，跑完再找
  const produced = fs
    .readdirSync(runDir)
    .filter((f) => f.endsWith('.json') && f !== 'meta.json');

  paths.writeJson(metaPath, {
    script: 'memory-eval',
    adapter: ADAPTER,
    dataset: DATASET,
    split: args.split || null,
    params: {
      config: CONFIG_NAME,
      sample: args.sample ? Number(args.sample) : null,
      seed: args.seed ? Number(args.seed) : null,
      stratified: Boolean(args.stratified),
      workers: WORKERS,
      top_k: TOP_K,
      link_weight: LINK_WEIGHT,
    },
    git: paths.gitCommit(),
    duration_s: Math.round((Date.now() - startedAt) / 1000),
    result_files: produced,
    error,
  });

  if (error) process.exit(1);
  console.log(`\n[eval] 产物：${runDir}`);
  for (const f of produced) console.log(`  ${f}`);
})().catch((e) => {
  console.error(`[eval] 未捕获异常：${e.message || e}`);
  process.exit(1);
});

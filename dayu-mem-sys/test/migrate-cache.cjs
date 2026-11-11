// 一次性迁移：旧的大 JSON 缓存 → 新的 JSONL 缓存
//
// 旧格式：test/data/cache/<profile>.json      {"<sha1(text)>": [float…], …}
// 新格式：test/data/cache/<backend>-<model>-<dim>-<dataset>-<sourceHash8>.jsonl
//
// 用法：
//   node test/migrate-cache.cjs               # 预览，不写文件
//   node test/migrate-cache.cjs --write       # 实际写入
//   node test/migrate-cache.cjs --write --keep-old
//
// 为什么值得迁移：远端嵌入按 token 计费，7554 条 2560 维重算一次是实打实的钱；
// 而旧缓存里的向量只要模型和维度对得上，就是有效数据。
//
// 迁移只搬「当前数据集用得到的文本」：旧缓存里历史残留的条目（改过的句子、
// 废弃的查询）不会被带过去——它们是垃圾，带过去只会让文件变大且无法判断来源。

const fs = require('fs');
const path = require('path');

const paths = require('./lib/paths.cjs');
const config = require('./lib/config.cjs');
const cacheLib = require('./lib/cache.cjs');
const datasetLib = require('./lib/dataset.cjs');
const { hashOf } = require('./lib/stats.cjs');

const WRITE = process.argv.includes('--write');
const KEEP_OLD = process.argv.includes('--keep-old');

const DATASET = (() => {
  const i = process.argv.indexOf('--dataset');
  return i >= 0 ? process.argv[i + 1] : 'main';
})();

/** 从配置文件推导 backend / model（与 harness.cacheIdentity 同一套规则） */
function identityOf(profile) {
  const toml = config.load(profile, {});
  const kv = config.keyValues(toml);
  return {
    profile,
    backend: kv.embedding_backend ?? profile,
    model: kv.embedding_api_model ?? kv.embedding_model_path ?? kv.embedding_model ?? profile,
  };
}

/** 旧 JSON 缓存里第一个向量的维度（不解析整个文件就能判断） */
function peekDim(file) {
  const fd = fs.openSync(file, 'r');
  try {
    const buf = Buffer.alloc(200_000);
    const n = fs.readSync(fd, buf, 0, buf.length, 0);
    const head = buf.toString('utf8', 0, n);
    const i = head.indexOf('[');
    const j = head.indexOf(']');
    if (i < 0 || j < 0) return null;
    return head.slice(i + 1, j).split(',').length;
  } finally {
    fs.closeSync(fd);
  }
}

function main() {
  const ds = datasetLib.load(DATASET);
  const wanted = new Map(); // sha1 → text
  for (const t of datasetLib.texts(ds, ds.memories.length)) {
    wanted.set(hashOf(t), t);
  }

  console.log(`数据集 ${ds.id}：需要 ${wanted.size} 条向量（记忆 ${ds.memories.length} + 查询 ${ds.queries.length}）\n`);

  const oldFiles = fs
    .readdirSync(paths.CACHE_DIR)
    .filter((f) => f.endsWith('.json'))
    .map((f) => path.join(paths.CACHE_DIR, f));

  if (!oldFiles.length) {
    console.log('没有旧的 .json 缓存，无需迁移。');
    return;
  }

  for (const oldFile of oldFiles) {
    const profile = path.basename(oldFile, '.json');
    let identity;
    try {
      identity = identityOf(profile);
    } catch (e) {
      console.log(`跳过 ${profile}：配置不可用（${e.message}）`);
      continue;
    }

    const dim = peekDim(oldFile);
    if (!dim) {
      console.log(`跳过 ${profile}：无法读取向量维度`);
      continue;
    }

    const target = paths.cachePath({ ...identity, dim }, ds);
    const oldBytes = fs.statSync(oldFile).size;

    let old;
    try {
      old = JSON.parse(fs.readFileSync(oldFile, 'utf8'));
    } catch (e) {
      console.log(`跳过 ${profile}：旧缓存解析失败（${e.message}）`);
      continue;
    }

    const hits = [];
    for (const [h] of wanted) {
      const v = old[h];
      if (Array.isArray(v) && v.length === dim) hits.push([h, v]);
    }

    console.log(`${profile} → ${path.basename(target)}`);
    console.log(
      `  模型 ${identity.model} / ${dim} 维 / 旧文件 ${(oldBytes / 1048576).toFixed(0)} MB / ` +
        `旧条目 ${Object.keys(old).length}`,
    );
    console.log(`  可迁移 ${hits.length} / ${wanted.size} 条`);

    if (hits.length < wanted.size) {
      const missing = [...wanted.keys()].filter((h) => !hits.some(([x]) => x === h));
      console.log(
        `  ⚠️ 缺 ${missing.length} 条，迁移后仍需调模型补齐：` +
          `「${wanted.get(missing[0]).slice(0, 24)}」…`,
      );
    }

    if (!WRITE) {
      console.log('  （预览模式，加 --write 才写文件）\n');
      continue;
    }

    // 已存在同名新缓存时先备份，避免把已有数据覆盖掉
    if (fs.existsSync(target)) {
      const bak = `${target}.bak-${paths.stamp()}`;
      fs.renameSync(target, bak);
      console.log(`  已有新缓存，备份为 ${path.basename(bak)}`);
    }

    cacheLib.open(target, {
      backend: identity.backend,
      model: identity.model,
      dim,
      dataset: ds.id,
      sourceHash: ds.sourceHash,
      preprocess: 'none',
      normalize: 'l2',
    });
    cacheLib.append(target, hits);

    const newBytes = fs.statSync(target).size;
    console.log(
      `  已写入 ${(newBytes / 1048576).toFixed(1)} MB（旧 ${(oldBytes / 1048576).toFixed(0)} MB）`,
    );

    if (!KEEP_OLD) {
      const oldBak = `${oldFile}.migrated-${paths.stamp()}`;
      fs.renameSync(oldFile, oldBak);
      console.log(`  旧文件改名为 ${path.basename(oldBak)}（确认无误后删除）`);
    }
    console.log('');
  }

  if (!WRITE) console.log('预览结束。加 --write 执行迁移。');
}

main();

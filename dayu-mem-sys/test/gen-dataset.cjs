// 数据集生成器：把人工语料铺成一份落地数据集
//
// 用法：
//   node test/gen-dataset.cjs                 # 默认 id=main，100000 条
//   node test/gen-dataset.cjs main 100000
//
// 产物：test/datasets/<id>.json（.gitignore，派生物不入库）
//
// 为什么要落地：
//   1. 测试读的是「同一份文件」，而不是每次现算——数据集的边界可见、可审
//   2. sourceHash 把「语料 + 展开规则 + 生成器」钉住，改了任一处就能被检测出陈旧
//   3. 向量缓存的键包含 sourceHash，重新生成 = 自动换代，不用手工清理
//
// 为什么确定性：本脚本不含任何随机数。同样的语料 + 同样的参数 → 逐字节相同的输出。
// 要做「另一个数据集」，就改 test/corpus.cjs 的人工语料，然后重跑本脚本。

const fs = require('fs');
const path = require('path');

const paths = require('./lib/paths.cjs');
const datasetLib = require('./lib/dataset.cjs');
const { DOMAINS, expandCorpus } = require('./lib/corpus.cjs');
const { QUERIES } = require('./corpus.cjs');

function generate(id, maxMemories) {
  const memories = expandCorpus(maxMemories).map((m) => ({
    text: m.text,
    domain: m.domain,
  }));

  return {
    id,
    sourceHash: datasetLib.sourceHash(),
    generator: {
      script: paths.rel(__filename),
      params: { maxMemories },
    },
    // 不写 createdAt：时间戳会让「同参数重跑产出逐字节相同」失效，
    // 而可复现是这份文件存在的理由。生成时间看文件 mtime 就够。
    counts: {
      memories: memories.length,
      queries: QUERIES.length,
      domains: DOMAINS.length,
    },
    domains: DOMAINS,
    memories,
    queries: QUERIES,
  };
}

// ---------- 主流程 ----------

const ID = process.argv[2] || 'main';
const MAX = Number(process.argv[3] || 100000);

if (!/^[a-z0-9][a-z0-9_-]*$/.test(ID)) {
  console.error(`数据集 id 非法：${ID}（只允许小写字母/数字/下划线/短横线）`);
  process.exit(1);
}
if (!Number.isInteger(MAX) || MAX <= 0) {
  console.error(`条数非法：${process.argv[3]}`);
  process.exit(1);
}

const t0 = Date.now();
const ds = generate(ID, MAX);
const outPath = path.join(datasetLib.DATASET_DIR, `${ID}.json`);

fs.mkdirSync(datasetLib.DATASET_DIR, { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(ds), 'utf8');

const bytes = fs.statSync(outPath).size;
console.log(`已生成 ${paths.rel(outPath)}`);
console.log(
  `  id=${ds.id}  sourceHash=${ds.sourceHash.slice(0, 8)}  ` +
    `记忆 ${ds.counts.memories} / 查询 ${ds.counts.queries} / 领域 ${ds.counts.domains}`,
);
console.log(`  ${(bytes / 1048576).toFixed(1)} MB，耗时 ${Date.now() - t0}ms`);
console.log(
  `\n向量缓存键：<backend>-<model>-<dim>-${ID}-${ds.sourceHash.slice(0, 8)}`,
);

// 校准「起始球去重」阈值：不同近似程度的句对余弦
//
// start_dedupe_threshold 该定在哪，取决于「同一件事的两次记录」与「两件不同的事」
// 在嵌入空间里的余弦分布。这个脚本只测句对，不建库。
//
// 运行：node test/lab/dedupe-threshold.cjs --profile http

const paths = require('../lib/paths.cjs');
const harness = require('../lib/harness.cjs');
const { cos } = require('../lib/stats.cjs');

const arg = (name, def) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : def;
};

const PROFILE = arg('profile', 'http');

const PAIRS = [
  ['完全副本（加口语前缀）', 'CI 上 clippy 开了 -D warnings，任何警告都过不了', '记一下：CI 上 clippy 开了 -D warnings，任何警告都过不了'],
  ['同义改写（换词）', '我喜欢吃辣', '我爱吃辣'],
  ['同义改写（口语/书面）', '这个功能上线三个月使用率不到百分之二', '该功能上线3个月使用率不足2%'],
  ['同事实不同细节', '炖牛肉要先焯水去血沫，再小火慢炖两个小时', '牛肉要焯水去沫，炖两个小时'],
  ['同主题不同角度', '跑步对膝盖冲击大', '游泳对膝盖友好'],
  ['同句换一个词（语义反转）', '我喜欢辣的食物', '我不喜欢辣的食物'],
  ['同词不同对象', '用户喜欢辣的食物', '用户喜欢甜的食物'],
  ['同领域不同内容', '订单表加了联合索引，查询变快', '订单表的主从延迟到了三秒'],
  ['同领域相关但独立', '跑步要热身', '跑步要选缓冲好的鞋'],
  ['跨领域无关', '订单表加了联合索引', '炖牛肉要先焯水'],
];

(async () => {
  const startedAt = new Date().toISOString();
  const run = paths.newRun('dedupe-threshold', PROFILE);
  const { api, dim, identity } = await harness.init({ profile: PROFILE, run });

  const texts = [...new Set(PAIRS.flatMap((p) => [p[1], p[2]]))];
  // 句对不来自数据集：用文本自身算 sourceHash，改了句子就换缓存
  const emb = await harness.embed(texts, {
    identity,
    dataset: harness.adhocDataset('dedupe-pairs', texts),
    chunk: 32,
  });
  api.shutdown();

  const vecOf = (t) => emb.vectors[texts.indexOf(t)];

  const rows = PAIRS.map(([label, a, b]) => ({ label, a, b, cos: cos(vecOf(a), vecOf(b)) }));
  rows.sort((x, y) => y.cos - x.cos);

  console.log(`=== 句对余弦（${PROFILE}，dim=${dim}）===`);
  console.log('类别'.padEnd(24) + ' | 余弦');
  for (const r of rows) {
    console.log(`${r.label.padEnd(24)} | ${r.cos.toFixed(4)}`);
  }

  // 阈值建议：完全副本与同义改写应当被判为「同一件事」，
  // 同主题不同角度以下不应被判为同一件事。取两者之间的空档。
  const dup = rows.filter((r) => /完全副本|同义改写/.test(r.label));
  const diff = rows.filter((r) => /同主题不同角度|同领域|跨领域|语义反转|不同对象/.test(r.label));
  const minDup = Math.min(...dup.map((r) => r.cos));
  const maxDiff = Math.max(...diff.map((r) => r.cos));
  console.log(
    `\n重复类最低余弦 ${minDup.toFixed(4)}，不同类最高余弦 ${maxDiff.toFixed(4)}，` +
      `空档 ${(minDup - maxDiff).toFixed(4)}`,
  );
  console.log(
    minDup > maxDiff
      ? `建议 start_dedupe_threshold ∈ (${maxDiff.toFixed(3)}, ${minDup.toFixed(3)}]`
      : `⚠️ 两类有重叠，单一阈值无法完全区分，需按实际取舍（当前 start_dedupe_threshold 见 config/base.toml）`,
  );

  paths.writeJson(run.resultPath, rows);
  harness.writeMeta(run, {
    script: 'dedupe-threshold',
    tag: PROFILE,
    profile: PROFILE,
    startedAt,
    dim,
    cache: emb.cachePath,
  });
  console.log(`\n产物：${paths.rel(run.dir)}`);
})().catch((e) => {
  console.error('FAILED', e.message);
  process.exit(1);
});

// 连通性验证：只调一次 HTTP 嵌入，确认 base/key/model 与维度
//
// 运行：node test/probe.cjs

const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..');
process.chdir(repoRoot);

const api = require(path.join(__dirname, 'dayu_test.node'));
const toml = fs.readFileSync(path.join(__dirname, 'config-http.toml'), 'utf8');

const dbPath = 'test/probe.tdb';
for (const f of fs.readdirSync('test')) {
  if (f.startsWith('probe.tdb')) fs.unlinkSync(path.join('test', f));
}

const t0 = Date.now();
api.init(toml.replace(/^db_path\s*=.*$/m, `db_path = "${dbPath}"`));
console.log(`init 成功：${Date.now() - t0}ms`);

(async () => {
  const stats = await api.getStats();
  console.log(`embeddingDim = ${stats.embeddingDim}`);

  const t = Date.now();
  const ballId = await api.createBall('用户的生日是 3 月 5 日', null, 3.0);
  console.log(`建球（含一次远程嵌入）：${Date.now() - t}ms，id = ${ballId}`);

  const t2 = Date.now();
  const r = await api.recall('用户的生日是哪天', 1.0);
  console.log(`回忆：${Date.now() - t2}ms，注入块 ${r.blocks.length} 个，token ${r.stats.tokenCount}`);
  console.log(
    `  点亮 ${r.stats.litCount}，起始 ${r.stats.startCount}，被挡 ${r.stats.blockedPipeCount}，强化 ${r.stats.strengthenedCount}`,
  );
  for (const b of r.blocks) {
    console.log(`  - [${b.confidence}] ${b.text.slice(0, 60)}`);
  }

  api.shutdown();
})().catch((e) => {
  console.error('FAILED', e);
  process.exit(1);
});

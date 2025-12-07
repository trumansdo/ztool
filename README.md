- 唯一库包：src/lib.rs
- 默认二进制包：src/main.rs，编译后生成的可执行文件与 Package 同名
- 其余二进制包：src/bin/main1.rs 和 src/bin/main2.rs，它们会分别生成一个文件同名的二进制可执行文件
- 集成测试文件：tests 目录下
- 基准性能测试 benchmark 文件：benches 目录下
- 项目示例：examples 目录下

模块化：

参考文档：
1. https://blog.csdn.net/wowotuo/article/details/107591501
2. https://blog.csdn.net/jiaoyangwm/article/details/136267564

在rust中一个文件或者目录都会被当成模块，但目录当成模块的方式与文件不一样。如果一个rs文件与main.rs/lib.rs同级的话，直接在main.rs/lib.rs中引入该模块即可。但目录就有所区别，目录的模块化有两种方法：
1. 在目录里创建一个 mod.rs，如果你使用的 rustc 版本 1.30 之前，这是唯一的方法。
2. 在目录的同级目录里创建一个与模块（目录）同名的 rs 文件，充当mod.rs，其内容与mod.rs一样，在新版本里，更建议使用这样的命名方式来避免项目中存在大量同名的 mod.rs 文件（ Python 点了个 踩）。

GUI选型：
https://blog.csdn.net/jjhenda00/article/details/155137478

综合技术选型：
gui库：egui的eframe
错误处理：anyhow+thiserror
异步运行时：tokio
日志：tracing
数据访问：Rbatis
http库：reqwest
配置管理: config
序列化: serde
时间日期: chrono
测试：tokio-test
通用工具和辅助功能：
  uuid/regex/once_cell/rand/fancy-regex
  tempfile/indexmap/itertools
  clap/walkdir/rayon
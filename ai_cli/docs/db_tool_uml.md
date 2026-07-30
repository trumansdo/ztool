# db_tool 架构设计 UML 文档

> 生成时间：2026-07-29  
> 项目：ztool/ai_cli  
> 工具：db_tool —— 数据库操作 CLI 工具

---

## 目录

1. [一、框架设计 UML](#一框架设计-uml)
   - [1.1 系统上下文图](#11-系统上下文图)
   - [1.2 分层架构图](#12-分层架构图)
   - [1.3 模块依赖图](#13-模块依赖图)
   - [1.4 部署架构图](#14-部署架构图)
2. [二、细节设计 UML](#二细节设计-uml)
   - [2.1 核心类图](#21-核心类图)
   - [2.2 CLI 命令解析类图](#22-cli-命令解析类图)
   - [2.3 配置体系类图](#23-配置体系类图)
   - [2.4 数据类型类图](#24-数据类型类图)
   - [2.5 错误体系类图](#25-错误体系类图)
   - [2.6 query 命令时序图](#26-query-命令时序图)
   - [2.7 execute 命令时序图](#27-execute-命令时序图)
   - [2.8 struct 命令时序图](#28-struct-命令时序图)
   - [2.9 连接建立活动图](#29-连接建立活动图)
   - [2.10 状态机图](#210-状态机图)

---

## 一、框架设计 UML

### 1.1 系统上下文图

```mermaid
C4Context
    title db_tool 系统上下文图

    Person(user, "开发者/运维", "通过命令行操作数据库")

    System(db_tool, "db_tool CLI", "数据库操作命令行工具")

    System_Ext(oracle_db, "Oracle 数据库", "IPM/IPM_TEST/INDEX 等")
    System_Ext(filesystem, "文件系统", "binconfig.toml 配置")
    System_Ext(oci, "Oracle Instant Client", "多版本共存 .dll/.so")

    Rel(user, db_tool, "执行子命令", "CLI args")
    Rel(db_tool, filesystem, "读取配置", "TOML")
    Rel(db_tool, oci, "动态加载", "OCI 接口")
    Rel(db_tool, oracle_db, "SQL 执行", "TCP 1521")
```

### 1.2 分层架构图

```mermaid
graph TB
    subgraph "CLI 入口层 (bin/)"
        A[db_tool.rs<br/>clap 参数解析<br/>main 入口]
    end

    subgraph "公共 API 层 (lib.rs)"
        B1[run_db_list]
        B2[run_db_query]
        B3[run_db_execute]
        B4[run_db_struct]
        B5[run_db_tables]
        B6[run_db_init_config]
        B7[format_and_print]
    end

    subgraph "数据库管理层 (db.rs)"
        C[DatabaseManager<br/>连接器注册<br/>操作路由]
    end

    subgraph "连接器抽象层 (db/connector.rs)"
        D1[DbConnector trait<br/>工厂接口]
        D2[DbConnection trait<br/>会话接口]
    end

    subgraph "数据库实现层 (db/oracle.rs)"
        E1[OracleConnector<br/>连接工厂]
        E2[OracleConnection<br/>SQL 执行<br/>事务管理]
    end

    subgraph "配置层 (db/config.rs)"
        F1[BinConfig<br/>配置加载]
        F2[DbConnectionConfig<br/>连接参数]
    end

    subgraph "类型层 (db/types.rs)"
        G1[QueryResult]
        G2[TableInfo]
        G3[ColumnInfo]
        G4[IndexInfo]
        G5[OutputFormat]
    end

    subgraph "错误层 (error.rs)"
        H[AiCliError<br/>统一错误类型]
    end

    A --> B1 & B2 & B3 & B4 & B5 & B6
    B2 --> B7
    B1 & B2 & B3 & B4 & B5 & B6 --> C
    C --> D1
    D1 --> E1
    E1 --> E2
    D2 --> E2
    C --> F1
    F1 --> F2
    B2 & B4 & B5 --> G1 & G2 & G3 & G4
    C --> H
```

### 1.3 模块依赖图

```mermaid
graph LR
    subgraph "bin"
        db_tool["db_tool.rs"]
    end

    subgraph "ai_cli lib"
        lib["lib.rs<br/>公共 API"]
        db["db.rs<br/>DatabaseManager"]
        connector["db/connector.rs<br/>trait 定义"]
        oracle["db/oracle.rs<br/>Oracle 实现"]
        config["db/config.rs<br/>配置加载"]
        types["db/types.rs<br/>类型定义"]
        error["error.rs<br/>错误类型"]
    end

    subgraph "外部依赖"
        clap["clap<br/>CLI 解析"]
        oracle_crate["oracle<br/>Oracle 驱动"]
        serde["serde<br/>序列化"]
        toml["toml<br/>配置解析"]
        tracing["tracing<br/>日志"]
    end

    db_tool --> lib
    db_tool --> clap
    lib --> db
    lib --> types
    lib --> error
    db --> connector
    db --> oracle
    db --> config
    db --> types
    db --> error
    oracle --> connector
    oracle --> config
    oracle --> types
    oracle --> error
    oracle --> oracle_crate
    config --> error
    config --> serde
    config --> toml
    types --> serde
    types --> clap
    error --> serde
```

### 1.4 部署架构图

```mermaid
graph TB
    subgraph "开发者机器"
        EXE["db_tool.exe<br/>(release build)"]
        CFG["binconfig.toml<br/>(exe 同级目录)"]
        OCI["Oracle Instant Client<br/>D:/program/instantclient/..."]
    end

    subgraph "网络"
        FW["防火墙/代理"]
    end

    subgraph "数据中心"
        subgraph "DEV 环境"
            DEV_DB[("IPM 开发库<br/>10.89.188.29:1521<br/>service: trade")]
        end
        subgraph "TEST 环境"
            TEST_DB[("IPM 测试库<br/>10.89.190.103:1521<br/>service: ipmdb")]
        end
        subgraph "INDEX 环境"
            IDX_DB[("INDEX 开发库<br/>10.89.188.77:1521<br/>service: trade")]
        end
    end

    EXE -->|读取| CFG
    EXE -->|动态加载| OCI
    EXE -->|TCP:1521| FW
    FW --> DEV_DB
    FW --> TEST_DB
    FW --> IDX_DB
```

---

## 二、细节设计 UML

### 2.1 核心类图

```mermaid
classDiagram
    class DatabaseManager {
        -config: BinConfig
        -connectors: HashMap~String, Box~dyn DbConnector~~
        +new() Result~Self~
        +list_databases() Vec~DbSummary~
        +execute_query(db_name, sql, limit) Result~QueryResult~
        +execute_dml(db_name, sqls) Result~u64~
        +get_table_struct(db_name, owner, table) Result~TableInfo~
        +list_tables(db_name, owner) Result~Vec~String~~
        -find_database(name) Result~&DbConnectionConfig~
        -get_connector(db_type) Result~&dyn DbConnector~
    }

    class DbConnector {
        <<interface>>
        +connect(config) Result~Box~dyn DbConnection~~
        +db_type() &'static str
    }

    class DbConnection {
        <<interface>>
        +execute_query(sql, max_rows) Result~QueryResult~
        +execute_dml(sqls) Result~u64~
        +get_table_struct(owner, table) Result~TableInfo~
        +list_tables(owner) Result~Vec~String~~
    }

    class OracleConnector {
        +connect(config) Result~Box~dyn DbConnection~~
        +db_type() &'static str
    }

    class OracleConnection {
        -inner: Connection
        -db_name: String
        +open(config) Result~Self~
        -row_str(row, idx) String
        -opt_row_str(row, idx) Option~String~
        +execute_query(sql, max_rows) Result~QueryResult~
        +execute_dml(sqls) Result~u64~
        +get_table_struct(owner, table) Result~TableInfo~
        +list_tables(owner) Result~Vec~String~~
    }

    DatabaseManager --> DbConnector : uses
    DatabaseManager --> DbConnection : creates via
    DbConnector <|.. OracleConnector : implements
    DbConnection <|.. OracleConnection : implements
    OracleConnector --> OracleConnection : creates
```

### 2.2 CLI 命令解析类图

```mermaid
classDiagram
    class DbToolCli {
        +format: OutputFormat
        +command: Commands
    }

    class Commands {
        <<enumeration>>
        List
        Query { db_name, sql, limit }
        Execute { db_name, sql }
        TableStruct { db_name, table, owner }
        Tables { db_name, owner }
        Init
    }

    class OutputFormat {
        <<enumeration>>
        Json
        Csv
        Table
    }

    DbToolCli --> Commands
    DbToolCli --> OutputFormat

    note for Commands "Query: -n/--limit 默认100\nExecute: --sql 可多次指定\nTableStruct: owner 默认空→当前用户\nTables: owner 可选"
```

### 2.3 配置体系类图

```mermaid
classDiagram
    class BinConfig {
        +db_tool: DbToolConfig
        +web_fetch: WebFetchConfig
        +load() Result~Self~
        +config_path() Result~PathBuf~
        +find_database(name) Option~&DbConnectionConfig~
        +summaries() Vec~DbSummary~
    }

    class DbToolConfig {
        +output_format: String
        +databases: Vec~DbConnectionConfig~
    }

    class WebFetchConfig {
        +http_proxy: Option~String~
    }

    class DbConnectionConfig {
        +name: String
        +desc: String
        +type: String
        +env: String
        +user: String
        +password: String
        +host: String
        +port: u16
        +service_name: String
        +lib_dir: Option~String~
        +url: Option~String~
        +extra_params: HashMap~String, String~
    }

    BinConfig *-- DbToolConfig
    BinConfig *-- WebFetchConfig
    DbToolConfig *-- DbConnectionConfig : 0..*

    note for DbConnectionConfig "lib_dir: Oracle 多版本共存关键\nurl: 直接连接字符串，优先于 host/port"
```

### 2.4 数据类型类图

```mermaid
classDiagram
    class QueryResult {
        +columns: Vec~String~
        +rows: Vec~Vec~String~~
        +row_count: usize
        +truncated: bool
    }

    class TableInfo {
        +table_name: String
        +owner: String
        +comment: Option~String~
        +columns: Vec~ColumnInfo~
        +indexes: Vec~IndexInfo~
    }

    class ColumnInfo {
        +column_id: u32
        +column_name: String
        +comment: Option~String~
        +nullable: String
        +data_type: String
        +data_length: Option~i64~
        +data_precision: Option~i64~
        +data_scale: Option~i64~
        +data_default: Option~String~
    }

    class IndexInfo {
        +index_name: String
        +column_name: String
        +column_position: u32
        +index_type: Option~String~
        +uniqueness: Option~String~
    }

    class DbSummary {
        +name: String
        +desc: String
        +db_type: String
        +env: String
        +user: String
    }

    TableInfo *-- ColumnInfo : 0..*
    TableInfo *-- IndexInfo : 0..*
```

### 2.5 错误体系类图

```mermaid
classDiagram
    class AiCliError {
        <<enumeration>>
        Http(reqwest::Error)
        Browser(String)
        Url(url::ParseError)
        Extraction(String)
        Io(std::io::Error)
        Json(serde_json::Error)
        Config(String)
        AiApi(String)
        General(String)
    }

    class Result~T~ {
        <<type alias>>
    }

    AiCliError --> Result : type Result~T~ = Result~T, AiCliError~

    note for AiCliError "使用 thiserror 派生\n支持 From 自动转换\n? 运算符无缝集成"
```

### 2.6 query 命令时序图

```mermaid
sequenceDiagram
    actor User
    participant CLI as db_tool main
    participant Lib as lib::run_db_query
    participant Mgr as DatabaseManager
    participant CFG as BinConfig
    participant OC as OracleConnector
    participant OConn as OracleConnection
    participant DB as Oracle DB

    User->>CLI: db_tool query IPM_TEST "SELECT ..." -n 100
    CLI->>CLI: clap parse → Commands::Query
    CLI->>Lib: run_db_query("IPM_TEST", sql, 100, format)
    Lib->>Mgr: new()
    Mgr->>CFG: load()
    CFG-->>Mgr: BinConfig
    Mgr->>Mgr: 注册 OracleConnector
    Mgr-->>Lib: DatabaseManager
    Lib->>Mgr: execute_query("IPM_TEST", sql, 100)
    Mgr->>Mgr: find_database("IPM_TEST")
    Mgr->>Mgr: get_connector("oracle")
    Mgr->>OC: connect(config)
    OC->>OConn: open(config)
    OConn->>OConn: prepend_path(lib_dir)
    OConn->>DB: Connection::connect(user, pwd, host:port/service)
    DB-->>OConn: Connection
    OConn-->>OC: OracleConnection
    OC-->>Mgr: Box<dyn DbConnection>
    Mgr->>OConn: execute_query(sql, 100)
    OConn->>DB: query(sql, &[])
    DB-->>OConn: ResultSet
    OConn->>OConn: 提取 column_info
    loop 逐行读取 (最多100行)
        OConn->>DB: next row
        DB-->>OConn: Row
        OConn->>OConn: row_str() 转换
    end
    OConn-->>Mgr: QueryResult { columns, rows, row_count, truncated }
    Mgr-->>Lib: QueryResult
    Lib->>Lib: format_and_print(result, format)
    Lib-->>CLI: Ok(())
    CLI-->>User: JSON/CSV/Table 输出
```

### 2.7 execute 命令时序图

```mermaid
sequenceDiagram
    actor User
    participant CLI as db_tool main
    participant Lib as lib::run_db_execute
    participant Mgr as DatabaseManager
    participant OConn as OracleConnection
    participant DB as Oracle DB

    User->>CLI: db_tool execute IPM_TEST --sql "INSERT..." --sql "UPDATE..."
    CLI->>CLI: clap parse → Commands::Execute { sql: ["INSERT...", "UPDATE..."] }
    CLI->>Lib: run_db_execute("IPM_TEST", &["INSERT...", "UPDATE..."])
    Lib->>Mgr: new() → load config
    Lib->>Mgr: execute_dml("IPM_TEST", &["INSERT...", "UPDATE..."])
    Mgr->>Mgr: find_database + get_connector
    Mgr->>OConn: connect → OracleConnection
    Mgr->>OConn: execute_dml(&["INSERT...", "UPDATE..."])

    loop 逐条执行
        OConn->>DB: execute("INSERT...", &[])
        DB-->>OConn: Statement (rows=1)
        OConn->>OConn: total += 1
        OConn->>DB: execute("UPDATE...", &[])
        DB-->>OConn: Statement (rows=1)
        OConn->>OConn: total += 1
    end

    OConn->>DB: commit()
    DB-->>OConn: OK
    OConn-->>Mgr: total = 2
    Mgr-->>Lib: 2
    Lib->>Lib: println!("{\"rows_affected\": 2}")
    Lib-->>CLI: Ok(())
    CLI-->>User: {"rows_affected": 2}
```

### 2.8 struct 命令时序图

```mermaid
sequenceDiagram
    actor User
    participant CLI as db_tool main
    participant Lib as lib::run_db_struct
    participant Mgr as DatabaseManager
    participant OConn as OracleConnection
    participant DB as Oracle DB

    User->>CLI: db_tool struct IPM_TEST T_CIPCFSYSRISKCONFIG
    CLI->>CLI: owner="" → None → unwrap_or("")
    CLI->>Lib: run_db_struct("IPM_TEST", "", "T_CIPCFSYSRISKCONFIG", format)
    Lib->>Mgr: get_table_struct("IPM_TEST", "", "T_CIPCFSYSRISKCONFIG")
    Mgr->>OConn: get_table_struct("", "T_CIPCFSYSRISKCONFIG")

    Note over OConn: owner 为空 → use_current_user = true

    OConn->>DB: SELECT USER FROM DUAL
    DB-->>OConn: "BBA"
    Note over OConn: actual_owner = "BBA"

    OConn->>DB: SELECT comments FROM user_tab_comments WHERE lower(table_name)=lower(:1)
    DB-->>OConn: "系统风险配置表"

    OConn->>DB: SELECT ... FROM user_tab_columns a JOIN user_tab_comments c ... LEFT JOIN user_col_comments cc ... WHERE lower(a.table_name)=lower(:1)
    DB-->>OConn: 9 rows (column info)

    OConn->>DB: SELECT ... FROM user_indexes i JOIN user_ind_columns t ... WHERE lower(t.table_name)=lower(:1)
    DB-->>OConn: index rows

    OConn-->>Mgr: TableInfo { owner="BBA", columns, indexes }
    Mgr-->>Lib: TableInfo
    Lib->>Lib: serde_json::to_string_pretty
    Lib-->>CLI: Ok(())
    CLI-->>User: JSON 表结构
```

### 2.9 连接建立活动图

```mermaid
stateDiagram-v2
    [*] --> 加载配置: DatabaseManager::new()
    加载配置 --> 检查配置文件: BinConfig::load()

    检查配置文件 --> 文件不存在: config_path 不存在
    检查配置文件 --> 解析TOML: 文件存在
    文件不存在 --> [*]: Err(Config)

    解析TOML --> 注册连接器: toml::from_str

    注册连接器 --> 就绪: connectors.insert("oracle", OracleConnector)

    就绪 --> 查找数据库: execute_* 调用
    查找数据库 --> 数据库不存在: find_database 失败
    查找数据库 --> 获取连接器: 找到配置
    数据库不存在 --> [*]: Err(Config)

    获取连接器 --> 类型不支持: get_connector 失败
    获取连接器 --> 创建连接: connector.connect(config)
    类型不支持 --> [*]: Err(Config)

    创建连接 --> 设置PATH: lib_dir 存在
    创建连接 --> 构建连接串: lib_dir 不存在
    设置PATH --> 构建连接串: prepend_path

    构建连接串 --> URL直连: url 字段非空
    构建连接串 --> host拼接: url 为空
    URL直连 --> 连接Oracle
    host拼接 --> 连接Oracle: "host:port/service_name"

    连接Oracle --> 连接失败: Connection::connect 失败
    连接Oracle --> 连接成功: OracleConnection 创建
    连接失败 --> [*]: Err(Config)
    连接成功 --> [*]: Box<dyn DbConnection>
```

### 2.10 状态机图

```mermaid
stateDiagram-v2
    [*] --> Idle: 程序启动

    state Idle {
        [*] --> 等待输入
    }

    state "命令分发" as Dispatch {
        等待输入 --> ListCmd: list
        等待输入 --> QueryCmd: query
        等待输入 --> ExecuteCmd: execute
        等待输入 --> StructCmd: struct
        等待输入 --> TablesCmd: tables
        等待输入 --> InitCmd: init
    }

    state "query 流程" as QueryFlow {
        QueryCmd --> 加载配置
        加载配置 --> 建立连接
        建立连接 --> 执行查询
        执行查询 --> 格式化输出
        格式化输出 --> 打印结果
    }

    state "execute 流程" as ExecuteFlow {
        ExecuteCmd --> 加载配置2
        加载配置2 --> 建立连接2
        建立连接2 --> 逐条执行SQL
        逐条执行SQL --> 统一Commit
        统一Commit --> 打印影响行数
    }

    state "struct 流程" as StructFlow {
        StructCmd --> 判断Owner
        判断Owner --> User视图: owner为空
        判断Owner --> All视图: owner非空
        User视图 --> 查询列信息
        All视图 --> 查询列信息
        查询列信息 --> 查询索引
        查询索引 --> 组装TableInfo
    }

    打印结果 --> Idle
    打印影响行数 --> Idle
    组装TableInfo --> 打印JSON
    打印JSON --> Idle
    ListCmd --> 打印列表
    打印列表 --> Idle
    TablesCmd --> 查询表列表
    查询表列表 --> 打印JSON2
    打印JSON2 --> Idle
    InitCmd --> 生成配置
    生成配置 --> Idle
```

---

## 附录：关键设计决策

| 决策                     | 原因                                           |
| ---------------------- | -------------------------------------------- |
| 每次操作新建连接，不用连接池         | Oracle 多版本共存需要动态切换 PATH，连接池难以管理              |
| owner 为空时用 `user_*` 视图 | `all_*` 视图 + `lower(owner)=lower('')` 永远匹配不到 |
| execute 统一事务 commit    | 批量 DML 需要原子性，避免部分成功                          |
| `--sql` 多值传参           | 比 JSON 数组更自然，shell 直接传参                      |
| `-n` 默认 100 行          | 防止大数据量表撑爆终端                                  |
| `lib_dir` 配置项          | 支持同一机器多个 Oracle 版本共存                         |
| trait 抽象连接器            | 为未来 MySQL/PostgreSQL 扩展预留接口                  |

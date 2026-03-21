# 第18章: Effect System / Reader パターン

## はじめに

「データベース接続」「設定」「ロガー」など、多くの関数が**共通の依存物**を必要とします。これらを引数で毎回渡すのは煩雑です。

**Reader パターン**はこの問題を関数型スタイルで解決します。「依存物を受け取り、値を返す関数」を一つの型として表現し、合成できるようにします。

**Effect System** はより広い概念で、「どんな副作用を起こすか」を型で表現します。Reader はその代表的な実装例です。

---

## 問題: 依存物を毎回渡す

```rust
struct Config {
    db_url: String,
    log_level: String,
}

fn connect(config: &Config) -> String {
    format!("connected to {}", config.db_url)
}

fn query(config: &Config, sql: &str) -> String {
    format!("[{}] query: {}", config.db_url, sql)
}

fn log(config: &Config, msg: &str) {
    if config.log_level == "debug" {
        println!("[DEBUG] {}", msg);
    }
}

fn process(config: &Config, input: &str) -> String {
    log(config, &format!("processing: {}", input));
    let conn = connect(config);
    let result = query(config, &format!("SELECT * FROM t WHERE v = '{}'", input));
    log(config, &format!("result: {}", result));
    format!("{}: {}", conn, result)
}
```

全関数が `config: &Config` を受け取っています。これが増えると面倒です。

---

## Reader パターン

`Reader<Env, A>` は「環境 `Env` を受け取り `A` を返す関数」をラップした型です。

```rust
pub struct Reader<Env, A> {
    run: Box<dyn Fn(&Env) -> A>,
}

impl<Env: 'static, A: 'static> Reader<Env, A> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Env) -> A + 'static,
    {
        Reader { run: Box::new(f) }
    }

    /// 環境を渡して実行する
    pub fn run_reader(&self, env: &Env) -> A {
        (self.run)(env)
    }

    /// 結果を変換する
    pub fn map<B: 'static, F>(self, f: F) -> Reader<Env, B>
    where
        F: Fn(A) -> B + 'static,
    {
        Reader::new(move |env| f((self.run)(env)))
    }

    /// 次の Reader を連鎖させる（Monad の bind）
    pub fn and_then<B: 'static, F>(self, f: F) -> Reader<Env, B>
    where
        F: Fn(A) -> Reader<Env, B> + 'static,
    {
        Reader::new(move |env| {
            let a = (self.run)(env);
            f(a).run_reader(env)
        })
    }
}
```

---

## Reader を使った依存性注入

```rust
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db_url: String,
    pub log_level: String,
    pub app_name: String,
}

/// 環境から DB URL を取得する Reader
pub fn get_db_url() -> Reader<AppConfig, String> {
    Reader::new(|cfg: &AppConfig| cfg.db_url.clone())
}

/// 環境から接続文字列を組み立てる Reader
pub fn connect() -> Reader<AppConfig, String> {
    Reader::new(|cfg: &AppConfig| {
        format!("[{}] connected to {}", cfg.app_name, cfg.db_url)
    })
}

/// クエリを実行する Reader
pub fn query(sql: impl Into<String> + 'static) -> Reader<AppConfig, String> {
    let sql = sql.into();
    Reader::new(move |cfg: &AppConfig| {
        format!("[{}] {}", cfg.db_url, sql)
    })
}

/// ログを記録する Reader（log_level が "debug" のときのみ出力）
pub fn log_message(msg: impl Into<String> + 'static) -> Reader<AppConfig, ()> {
    let msg = msg.into();
    Reader::new(move |cfg: &AppConfig| {
        if cfg.log_level == "debug" {
            println!("[DEBUG] {}", msg);
        }
    })
}
```

---

## Reader の合成

```rust
pub fn process_user(user_id: u32) -> Reader<AppConfig, String> {
    connect().and_then(move |conn| {
        query(format!("SELECT * FROM users WHERE id = {}", user_id))
            .and_then(move |result| {
                log_message(format!("found user: {}", result))
                    .map(move |_| format!("{} → {}", conn, result))
            })
    })
}

fn main() {
    let config = AppConfig {
        db_url: "postgres://localhost/mydb".to_string(),
        log_level: "debug".to_string(),
        app_name: "MyApp".to_string(),
    };

    // 実行時に環境を注入する
    let result = process_user(42).run_reader(&config);
    println!("{}", result);
}
```

`process_user` の中には `AppConfig` への直接参照がありません。**依存物は最後の `run_reader` 呼び出し時に一度だけ渡されます。**

---

## テスタビリティ: 環境の差し替え

Reader パターンの最大の利点は**テスト時に環境を差し替えられる**ことです。

```rust
fn main_or_test() {
    // 本番環境
    let prod_config = AppConfig {
        db_url: "postgres://prod-server/db".to_string(),
        log_level: "info".to_string(),
        app_name: "ProdApp".to_string(),
    };

    // テスト環境
    let test_config = AppConfig {
        db_url: "sqlite::memory:".to_string(),
        log_level: "debug".to_string(),
        app_name: "TestApp".to_string(),
    };

    // 同じプログラムを異なる環境で実行
    let prog = process_user(1);
    let result_prod = process_user(1).run_reader(&prod_config);
    let result_test = prog.run_reader(&test_config);

    println!("prod: {}", result_prod);
    println!("test: {}", result_test);
}
```

---

## トレイトによる Effect の表現

より実用的なアプローチとして、トレイトで「できること（Effect）」を定義し、実装を差し替えます。これが Rust における Effect System の最もシンプルな形です。

```rust
/// ロギングの効果を表すトレイト
pub trait Logger {
    fn log(&self, msg: &str);
}

/// データベースアクセスの効果を表すトレイト
pub trait Repository {
    fn find(&self, id: u32) -> Option<String>;
    fn save(&self, id: u32, value: &str);
}

/// 両方の効果を必要とする関数
pub fn process_with_traits<L: Logger, R: Repository>(
    logger: &L,
    repo: &R,
    id: u32,
    new_value: &str,
) -> Option<String> {
    logger.log(&format!("processing id={}", id));
    let old = repo.find(id);
    repo.save(id, new_value);
    logger.log(&format!("saved: {}", new_value));
    old
}

// --- 本番実装 ---
pub struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, msg: &str) { println!("[LOG] {}", msg); }
}

// --- テスト実装 ---
pub struct SilentLogger;
impl Logger for SilentLogger {
    fn log(&self, _msg: &str) {} // 何もしない
}

pub struct InMemoryRepo {
    pub data: std::cell::RefCell<std::collections::HashMap<u32, String>>,
}

impl InMemoryRepo {
    pub fn new() -> Self {
        InMemoryRepo { data: std::cell::RefCell::new(std::collections::HashMap::new()) }
    }
}

impl Repository for InMemoryRepo {
    fn find(&self, id: u32) -> Option<String> {
        self.data.borrow().get(&id).cloned()
    }
    fn save(&self, id: u32, value: &str) {
        self.data.borrow_mut().insert(id, value.to_string());
    }
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        let logger = SilentLogger;  // テスト中はログを出力しない
        let repo = InMemoryRepo::new();

        // 最初は None
        let old = process_with_traits(&logger, &repo, 1, "hello");
        assert_eq!(old, None);

        // 2回目は前の値を返す
        let old2 = process_with_traits(&logger, &repo, 1, "world");
        assert_eq!(old2, Some("hello".to_string()));
    }
}
```

---

## Reader vs トレイトの使い分け

| アプローチ | 向いている場面 |
|-----------|--------------|
| **Reader** | 設定や環境を多くの関数で共有したい。関数合成を多用する。 |
| **トレイト（DI）** | 複数の実装を差し替えたい。テスト時にモックを注入したい。 |
| **両方の組み合わせ** | 大規模なアプリケーションで、設定の共有とモックの差し替えを両立したい。 |

---

## まとめ

| 概念 | 役割 |
|------|------|
| `Reader<Env, A>` | 環境を引数に取る計算を一級の値として表現し、合成する |
| `and_then` | Reader を連鎖させ、環境を暗黙的に引き回す |
| `run_reader` | 最後に環境を一度だけ渡して実行する |
| トレイトベース DI | 効果をトレイトで抽象化し、本番/テスト実装を差し替える |

Reader パターンと Effect System は「副作用を型で制御する」関数型の核心的なアイデアです。Rust ではトレイトとジェネリクスを使った実装が実用的です。

---

## よくある落とし穴と対処法

**落とし穴1: `Fn` vs `FnOnce` の選択ミス**

`Reader` が `Fn`（複数回呼べる）ではなく `FnOnce` だと、`run_reader` を一度しか呼べません。`Reader` は複数回実行できる `Fn` で定義するのが自然です。

**落とし穴2: `RefCell` の多用**

`InMemoryRepo` のように内部可変性（`RefCell`）を使う場面では、`borrow_mut` のパニックに注意してください。シングルスレッドのテストでは問題になりにくいですが、マルチスレッドでは `Mutex` を使いましょう。

---

## 章末演習問題

1. `Reader<AppConfig, Vec<String>>` として「設定に基づいてバナーメッセージのリストを生成する」関数を実装してください。

2. `Logger` トレイトを実装する `VecLogger`（ログを `Vec<String>` に蓄積する）を作り、テスト内でどのメッセージがログされたかを検証してください。

3. `Reader<AppConfig, A>` に `zip` 関数（2つの Reader を並行して実行し、結果をタプルで返す）を実装してください：
```rust
fn zip<B: 'static>(self, other: Reader<Env, B>) -> Reader<Env, (A, B)>
```

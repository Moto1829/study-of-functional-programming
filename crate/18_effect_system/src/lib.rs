use std::cell::RefCell;
use std::collections::HashMap;

// --- Reader パターン ---

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

    pub fn run_reader(&self, env: &Env) -> A {
        (self.run)(env)
    }

    pub fn map<B: 'static, F>(self, f: F) -> Reader<Env, B>
    where
        F: Fn(A) -> B + 'static,
    {
        Reader::new(move |env| f((self.run)(env)))
    }

    pub fn and_then<B: 'static, F>(self, f: F) -> Reader<Env, B>
    where
        F: Fn(A) -> Reader<Env, B> + 'static,
    {
        Reader::new(move |env| {
            let a = (self.run)(env);
            f(a).run_reader(env)
        })
    }

    pub fn zip<B: 'static>(self, other: Reader<Env, B>) -> Reader<Env, (A, B)> {
        Reader::new(move |env| {
            let a = (self.run)(env);
            let b = other.run_reader(env);
            (a, b)
        })
    }
}

// --- AppConfig ---

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db_url: String,
    pub log_level: String,
    pub app_name: String,
}

pub fn get_db_url() -> Reader<AppConfig, String> {
    Reader::new(|cfg: &AppConfig| cfg.db_url.clone())
}

pub fn connect() -> Reader<AppConfig, String> {
    Reader::new(|cfg: &AppConfig| {
        format!("[{}] connected to {}", cfg.app_name, cfg.db_url)
    })
}

pub fn query(sql: impl Into<String> + 'static) -> Reader<AppConfig, String> {
    let sql = sql.into();
    Reader::new(move |cfg: &AppConfig| format!("[{}] {}", cfg.db_url, sql))
}

pub fn process_user(user_id: u32) -> Reader<AppConfig, String> {
    connect().and_then(move |conn| {
        query(format!("SELECT * FROM users WHERE id = {}", user_id))
            .map(move |result| format!("{} → {}", conn, result))
    })
}

// --- トレイトベース Effect System ---

pub trait Logger {
    fn log(&self, msg: &str);
}

pub trait Repository {
    fn find(&self, id: u32) -> Option<String>;
    fn save(&self, id: u32, value: &str);
}

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
    fn log(&self, msg: &str) {
        println!("[LOG] {}", msg);
    }
}

// --- テスト実装 ---

pub struct SilentLogger;
impl Logger for SilentLogger {
    fn log(&self, _msg: &str) {}
}

pub struct VecLogger {
    pub messages: RefCell<Vec<String>>,
}

impl VecLogger {
    pub fn new() -> Self {
        VecLogger { messages: RefCell::new(Vec::new()) }
    }

    pub fn entries(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }
}

impl Logger for VecLogger {
    fn log(&self, msg: &str) {
        self.messages.borrow_mut().push(msg.to_string());
    }
}

pub struct InMemoryRepo {
    pub data: RefCell<HashMap<u32, String>>,
}

impl InMemoryRepo {
    pub fn new() -> Self {
        InMemoryRepo { data: RefCell::new(HashMap::new()) }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig {
            db_url: "sqlite::memory:".to_string(),
            log_level: "debug".to_string(),
            app_name: "TestApp".to_string(),
        }
    }

    #[test]
    fn test_reader_run() {
        let config = test_config();
        let result = get_db_url().run_reader(&config);
        assert_eq!(result, "sqlite::memory:");
    }

    #[test]
    fn test_reader_map() {
        let config = test_config();
        let result = get_db_url().map(|url| url.len()).run_reader(&config);
        assert_eq!(result, "sqlite::memory:".len());
    }

    #[test]
    fn test_reader_and_then() {
        let config = test_config();
        let result = connect()
            .and_then(|conn| query("SELECT 1").map(move |q| format!("{} | {}", conn, q)))
            .run_reader(&config);
        assert!(result.contains("TestApp"));
        assert!(result.contains("SELECT 1"));
    }

    #[test]
    fn test_reader_zip() {
        let config = test_config();
        let (url, conn) = get_db_url().zip(connect()).run_reader(&config);
        assert_eq!(url, "sqlite::memory:");
        assert!(conn.contains("TestApp"));
    }

    #[test]
    fn test_process_user() {
        let config = test_config();
        let result = process_user(42).run_reader(&config);
        assert!(result.contains("42"));
    }

    #[test]
    fn test_same_reader_multiple_times() {
        let config = test_config();
        let reader = get_db_url();
        // 同じ Reader を複数回実行できる
        assert_eq!(reader.run_reader(&config), reader.run_reader(&config));
    }

    #[test]
    fn test_trait_process_first_call() {
        let logger = SilentLogger;
        let repo = InMemoryRepo::new();

        let old = process_with_traits(&logger, &repo, 1, "hello");
        assert_eq!(old, None);
        assert_eq!(repo.find(1), Some("hello".to_string()));
    }

    #[test]
    fn test_trait_process_second_call() {
        let logger = SilentLogger;
        let repo = InMemoryRepo::new();

        process_with_traits(&logger, &repo, 1, "first");
        let old = process_with_traits(&logger, &repo, 1, "second");

        assert_eq!(old, Some("first".to_string()));
        assert_eq!(repo.find(1), Some("second".to_string()));
    }

    #[test]
    fn test_vec_logger_captures_messages() {
        let logger = VecLogger::new();
        let repo = InMemoryRepo::new();

        process_with_traits(&logger, &repo, 5, "value");

        let entries = logger.entries();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("5"));
        assert!(entries[1].contains("value"));
    }

    #[test]
    fn test_different_envs_give_different_results() {
        let dev_config = AppConfig {
            db_url: "sqlite::memory:".to_string(),
            log_level: "debug".to_string(),
            app_name: "Dev".to_string(),
        };
        let prod_config = AppConfig {
            db_url: "postgres://prod/db".to_string(),
            log_level: "info".to_string(),
            app_name: "Prod".to_string(),
        };

        let dev_result = get_db_url().run_reader(&dev_config);
        let prod_result = get_db_url().run_reader(&prod_config);

        assert_ne!(dev_result, prod_result);
    }
}

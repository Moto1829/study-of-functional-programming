use std::collections::HashMap;

// --- ストレージ DSL ---

pub enum StorageOp<Next> {
    Get(String, Box<dyn FnOnce(Option<String>) -> Next>),
    Set(String, String, Box<dyn FnOnce() -> Next>),
    Delete(String, Box<dyn FnOnce() -> Next>),
}

pub enum Program<A> {
    Pure(A),
    Step(StorageOp<Program<A>>),
}

impl<A: 'static> Program<A> {
    pub fn and_then<B: 'static, F>(self, f: F) -> Program<B>
    where
        F: FnOnce(A) -> Program<B> + 'static,
    {
        match self {
            Program::Pure(a) => f(a),
            Program::Step(op) => Program::Step(match op {
                StorageOp::Get(k, next) => StorageOp::Get(
                    k,
                    Box::new(move |v| next(v).and_then(f)),
                ),
                StorageOp::Set(k, v, next) => StorageOp::Set(
                    k,
                    v,
                    Box::new(move || next().and_then(f)),
                ),
                StorageOp::Delete(k, next) => StorageOp::Delete(
                    k,
                    Box::new(move || next().and_then(f)),
                ),
            }),
        }
    }

    pub fn map<B: 'static, F>(self, f: F) -> Program<B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        self.and_then(move |a| Program::Pure(f(a)))
    }
}

// --- スマートコンストラクタ ---

pub fn get(key: impl Into<String>) -> Program<Option<String>> {
    Program::Step(StorageOp::Get(
        key.into(),
        Box::new(Program::Pure),
    ))
}

pub fn set(key: impl Into<String>, value: impl Into<String>) -> Program<()> {
    Program::Step(StorageOp::Set(
        key.into(),
        value.into(),
        Box::new(|| Program::Pure(())),
    ))
}

pub fn delete(key: impl Into<String>) -> Program<()> {
    Program::Step(StorageOp::Delete(
        key.into(),
        Box::new(|| Program::Pure(())),
    ))
}

// --- プログラムの記述 ---

/// from のキーを to にコピーして from を削除する
pub fn transfer_value(from: &'static str, to: &'static str) -> Program<bool> {
    get(from).and_then(move |from_val| match from_val {
        None => Program::Pure(false),
        Some(val) => set(to, val)
            .and_then(move |_| delete(from))
            .and_then(|_| Program::Pure(true)),
    })
}

// --- インタープリタ: 本番用 ---

pub fn run_in_memory<A>(program: Program<A>, store: &mut HashMap<String, String>) -> A {
    match program {
        Program::Pure(a) => a,
        Program::Step(op) => match op {
            StorageOp::Get(k, next) => {
                let v = store.get(&k).cloned();
                run_in_memory(next(v), store)
            }
            StorageOp::Set(k, v, next) => {
                store.insert(k, v);
                run_in_memory(next(), store)
            }
            StorageOp::Delete(k, next) => {
                store.remove(&k);
                run_in_memory(next(), store)
            }
        },
    }
}

// --- インタープリタ: ログ付き ---

pub fn run_with_log<A>(
    program: Program<A>,
    store: &mut HashMap<String, String>,
    log: &mut Vec<String>,
) -> A {
    match program {
        Program::Pure(a) => a,
        Program::Step(op) => match op {
            StorageOp::Get(k, next) => {
                let v = store.get(&k).cloned();
                log.push(format!("GET {} → {:?}", k, v));
                run_with_log(next(v), store, log)
            }
            StorageOp::Set(k, v, next) => {
                log.push(format!("SET {} = {}", k, v));
                store.insert(k, v);
                run_with_log(next(), store, log)
            }
            StorageOp::Delete(k, next) => {
                log.push(format!("DELETE {}", k));
                store.remove(&k);
                run_with_log(next(), store, log)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_existing_key() {
        let mut store = HashMap::new();
        store.insert("key".to_string(), "value".to_string());
        let result = run_in_memory(get("key"), &mut store);
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_get_missing_key() {
        let mut store = HashMap::new();
        let result = run_in_memory(get("missing"), &mut store);
        assert_eq!(result, None);
    }

    #[test]
    fn test_set_and_get() {
        let mut store = HashMap::new();
        let prog = set("k", "v").and_then(|_| get("k"));
        let result = run_in_memory(prog, &mut store);
        assert_eq!(result, Some("v".to_string()));
    }

    #[test]
    fn test_transfer_success() {
        let mut store = HashMap::new();
        store.insert("from".to_string(), "hello".to_string());

        let result = run_in_memory(transfer_value("from", "to"), &mut store);

        assert_eq!(result, true);
        assert_eq!(store.get("to"), Some(&"hello".to_string()));
        assert_eq!(store.get("from"), None);
    }

    #[test]
    fn test_transfer_missing_source() {
        let mut store = HashMap::new();
        let result = run_in_memory(transfer_value("from", "to"), &mut store);
        assert_eq!(result, false);
        assert!(store.is_empty());
    }

    #[test]
    fn test_run_with_log_records_operations() {
        let mut store = HashMap::new();
        store.insert("from".to_string(), "data".to_string());
        let mut log = Vec::new();

        run_with_log(transfer_value("from", "to"), &mut store, &mut log);

        assert_eq!(log.len(), 3);
        assert!(log[0].contains("GET from"));
        assert!(log[1].contains("SET to"));
        assert!(log[2].contains("DELETE from"));
    }

    #[test]
    fn test_same_result_with_different_interpreters() {
        let mut store1 = HashMap::new();
        store1.insert("x".to_string(), "42".to_string());
        let mut store2 = store1.clone();
        let mut log = Vec::new();

        let result1 = run_in_memory(transfer_value("x", "y"), &mut store1);
        let result2 = run_with_log(transfer_value("x", "y"), &mut store2, &mut log);

        // 同じプログラムを異なるインタープリタで実行しても同じ結果
        assert_eq!(result1, result2);
        assert_eq!(store1, store2);
    }

    #[test]
    fn test_program_is_pure_data() {
        // プログラムを構築してもストアは変化しない
        let mut store = HashMap::new();
        store.insert("k".to_string(), "v".to_string());

        // プログラムを構築するだけ（実行しない）
        let _prog = set("k", "changed").and_then(|_| get("k"));

        // ストアは変化していない
        assert_eq!(store.get("k"), Some(&"v".to_string()));
    }
}

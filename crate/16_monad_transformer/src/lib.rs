// --- transpose パターン ---

/// Option<Result<T, E>> → Result<Option<T>, E>
pub fn parse_optional_number(s: Option<&str>) -> Result<Option<i32>, std::num::ParseIntError> {
    s.map(|v| v.parse::<i32>()).transpose()
}

// --- Writer パターン ---

/// 計算結果と付随するログをペアで持つ型
#[derive(Debug, Clone)]
pub struct Writer<A> {
    pub value: A,
    pub log: Vec<String>,
}

impl<A> Writer<A> {
    pub fn new(value: A) -> Self {
        Writer { value, log: Vec::new() }
    }

    pub fn tell(mut self, message: impl Into<String>) -> Self {
        self.log.push(message.into());
        self
    }

    pub fn map<B, F>(self, f: F) -> Writer<B>
    where
        F: FnOnce(A) -> B,
    {
        Writer { value: f(self.value), log: self.log }
    }

    pub fn and_then<B, F>(self, f: F) -> Writer<B>
    where
        F: FnOnce(A) -> Writer<B>,
    {
        let mut result = f(self.value);
        let mut combined_log = self.log;
        combined_log.append(&mut result.log);
        Writer { value: result.value, log: combined_log }
    }
}

pub fn double_with_log(n: i32) -> Writer<i32> {
    Writer::new(n * 2).tell(format!("{} を2倍にした → {}", n, n * 2))
}

pub fn add_ten_with_log(n: i32) -> Writer<i32> {
    Writer::new(n + 10).tell(format!("{} に10を足した → {}", n, n + 10))
}

// --- State パターン ---

/// 状態 S を受け取り、(結果 A, 新しい状態 S) を返す関数をラップした型
pub struct State<S, A> {
    pub run: Box<dyn FnOnce(S) -> (A, S)>,
}

impl<S: 'static, A: 'static> State<S, A> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(S) -> (A, S) + 'static,
    {
        State { run: Box::new(f) }
    }

    pub fn run_state(self, s: S) -> (A, S) {
        (self.run)(s)
    }

    pub fn map<B: 'static, F>(self, f: F) -> State<S, B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        State::new(move |s| {
            let (a, s2) = self.run_state(s);
            (f(a), s2)
        })
    }

    pub fn and_then<B: 'static, F>(self, f: F) -> State<S, B>
    where
        F: FnOnce(A) -> State<S, B> + 'static,
    {
        State::new(move |s| {
            let (a, s2) = self.run_state(s);
            f(a).run_state(s2)
        })
    }
}

pub fn get<S: Clone + 'static>() -> State<S, S> {
    State::new(|s: S| (s.clone(), s))
}

pub fn put<S: 'static>(new_state: S) -> State<S, ()> {
    State::new(|_| ((), new_state))
}

// --- エラー型統合パターン ---

#[derive(Debug, PartialEq)]
pub enum AppError {
    Parse(String),
    Logic(String),
    NotFound,
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self {
        AppError::Parse(e.to_string())
    }
}

pub fn parse_positive(s: &str) -> Result<u32, AppError> {
    let n: i32 = s.parse()?;
    if n < 0 {
        return Err(AppError::Logic(format!("{} は負の数", n)));
    }
    Ok(n as u32)
}

pub fn lookup_and_parse(data: &[(&str, &str)], key: &str) -> Result<u32, AppError> {
    let value = data.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .ok_or(AppError::NotFound)?;
    parse_positive(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpose_none() {
        assert_eq!(parse_optional_number(None), Ok(None));
    }

    #[test]
    fn test_transpose_some_ok() {
        assert_eq!(parse_optional_number(Some("42")), Ok(Some(42)));
    }

    #[test]
    fn test_transpose_some_err() {
        assert!(parse_optional_number(Some("abc")).is_err());
    }

    #[test]
    fn test_writer_and_then() {
        let result = Writer::new(5)
            .and_then(double_with_log)
            .and_then(add_ten_with_log);

        assert_eq!(result.value, 20);
        assert_eq!(result.log.len(), 2);
        assert!(result.log[0].contains("2倍"));
        assert!(result.log[1].contains("10を足した"));
    }

    #[test]
    fn test_writer_map() {
        let result = Writer::new(3).tell("start").map(|n| n * 2);
        assert_eq!(result.value, 6);
        assert_eq!(result.log, vec!["start"]);
    }

    #[test]
    fn test_state_get_put() {
        let computation = get::<i32>()
            .and_then(|n| put(n + 1).map(move |_| n));

        let (old_value, new_state) = computation.run_state(10);
        assert_eq!(old_value, 10);
        assert_eq!(new_state, 11);
    }

    #[test]
    fn test_state_sequence() {
        let inc = || {
            get::<i32>().and_then(|n| put(n + 1).map(move |_| n + 1))
        };
        let computation = inc().and_then(move |_| inc()).and_then(move |_| inc());
        let (result, state) = computation.run_state(0);
        assert_eq!(result, 3);
        assert_eq!(state, 3);
    }

    #[test]
    fn test_app_error_integration() {
        let data = vec![("age", "25"), ("score", "-5"), ("missing_key", "abc")];

        assert_eq!(lookup_and_parse(&data, "age"), Ok(25));
        assert!(matches!(
            lookup_and_parse(&data, "score"),
            Err(AppError::Logic(_))
        ));
        assert_eq!(lookup_and_parse(&data, "not_found"), Err(AppError::NotFound));
        assert!(matches!(
            lookup_and_parse(&data, "missing_key"),
            Err(AppError::Parse(_))
        ));
    }
}

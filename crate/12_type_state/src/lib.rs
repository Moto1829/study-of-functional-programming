// 第12章: 型状態パターン

use std::marker::PhantomData;

// ─── 状態を表すマーカー型 ─────────────────────────────────────

pub struct Locked;
pub struct Unlocked;

// ─── 型状態パターンによるドアの例 ────────────────────────────

/// PhantomData で状態を型パラメータとして持つドア
pub struct Door<State> {
    name: String,
    _state: PhantomData<State>,
}

impl Door<Locked> {
    /// 新しいドアは必ずロック状態で生成
    pub fn new(name: impl Into<String>) -> Self {
        Door {
            name: name.into(),
            _state: PhantomData,
        }
    }

    /// ロック状態でのみ unlock() が呼べる
    pub fn unlock(self) -> Door<Unlocked> {
        Door {
            name: self.name,
            _state: PhantomData,
        }
    }
}

impl Door<Unlocked> {
    /// アンロック状態でのみ open() が呼べる
    pub fn open(&self) -> String {
        format!("{} is open!", self.name)
    }

    /// アンロック状態でのみ lock() が呼べる
    pub fn lock(self) -> Door<Locked> {
        Door {
            name: self.name,
            _state: PhantomData,
        }
    }
}

// ─── ビルダーパターンへの応用 ─────────────────────────────────

pub struct NoEmail;
pub struct HasEmail;
pub struct NoName;
pub struct HasName;

/// 型状態を使った安全なビルダー
pub struct UserBuilder<E, N> {
    email: Option<String>,
    name: Option<String>,
    _email_state: PhantomData<E>,
    _name_state: PhantomData<N>,
}

impl UserBuilder<NoEmail, NoName> {
    pub fn new() -> Self {
        UserBuilder {
            email: None,
            name: None,
            _email_state: PhantomData,
            _name_state: PhantomData,
        }
    }
}

impl Default for UserBuilder<NoEmail, NoName> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> UserBuilder<NoEmail, N> {
    pub fn email(self, email: impl Into<String>) -> UserBuilder<HasEmail, N> {
        UserBuilder {
            email: Some(email.into()),
            name: self.name,
            _email_state: PhantomData,
            _name_state: PhantomData,
        }
    }
}

impl<E> UserBuilder<E, NoName> {
    pub fn name(self, name: impl Into<String>) -> UserBuilder<E, HasName> {
        UserBuilder {
            email: self.email,
            name: Some(name.into()),
            _email_state: PhantomData,
            _name_state: PhantomData,
        }
    }
}

/// email と name が両方セットされた場合のみ build() できる
impl UserBuilder<HasEmail, HasName> {
    pub fn build(self) -> User {
        User {
            email: self.email.unwrap(),
            name: self.name.unwrap(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct User {
    pub email: String,
    pub name: String,
}

// ─── HTTP リクエストの状態機械 ────────────────────────────────

pub struct Idle;
pub struct Pending;
pub struct Complete;

pub struct Request<State> {
    url: String,
    body: Option<String>,
    response: Option<String>,
    _state: PhantomData<State>,
}

impl Request<Idle> {
    pub fn new(url: impl Into<String>) -> Self {
        Request {
            url: url.into(),
            body: None,
            response: None,
            _state: PhantomData,
        }
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn send(self) -> Request<Pending> {
        Request {
            url: self.url,
            body: self.body,
            response: None,
            _state: PhantomData,
        }
    }
}

impl Request<Pending> {
    /// レスポンスを受信して Complete 状態に遷移
    pub fn receive(self, response: impl Into<String>) -> Request<Complete> {
        Request {
            url: self.url,
            body: self.body,
            response: Some(response.into()),
            _state: PhantomData,
        }
    }
}

impl Request<Complete> {
    /// Complete 状態でのみレスポンスを取得できる
    pub fn response(&self) -> &str {
        self.response.as_deref().unwrap()
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_door_state_machine() {
        let door = Door::new("Front Door");
        // door.open() // コンパイルエラー: Locked 状態では open() できない
        let door = door.unlock();
        assert_eq!(door.open(), "Front Door is open!");
        let _locked = door.lock();
        // _locked.open() // コンパイルエラー: 再びロックされた
    }

    #[test]
    fn test_builder_with_type_state() {
        let user = UserBuilder::new()
            .email("alice@example.com")
            .name("Alice")
            .build();

        assert_eq!(user, User {
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
        });
    }

    #[test]
    fn test_builder_order_independence() {
        // name → email の順でも build() できる
        let user = UserBuilder::new()
            .name("Bob")
            .email("bob@example.com")
            .build();

        assert_eq!(user.name, "Bob");
        assert_eq!(user.email, "bob@example.com");
    }

    #[test]
    fn test_request_state_machine() {
        let request = Request::new("https://example.com/api")
            .with_body(r#"{"key": "value"}"#)
            .send();

        // request.response() // コンパイルエラー: Pending 状態では取得できない

        let completed = request.receive("200 OK");
        assert_eq!(completed.response(), "200 OK");
        assert_eq!(completed.url(), "https://example.com/api");
    }
}

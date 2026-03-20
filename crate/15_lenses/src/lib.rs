/// Lens: 構造体のフィールドへの get/set を一級の値として表現する
pub struct Lens<S, A> {
    pub get: fn(&S) -> A,
    pub set: fn(S, A) -> S,
}

impl<S: Clone, A: Clone> Lens<S, A> {
    /// フィールドの値を取得する
    pub fn view(&self, s: &S) -> A {
        (self.get)(s)
    }

    /// フィールドの値を新しい値で置き換えた新しい S を返す
    pub fn update(&self, s: S, a: A) -> S {
        (self.set)(s, a)
    }

    /// フィールドの値を関数で変換した新しい S を返す
    pub fn modify<F>(&self, s: S, f: F) -> S
    where
        F: FnOnce(A) -> A,
    {
        let a = (self.get)(&s);
        self.update(s, f(a))
    }
}

// --- サンプルデータ型 ---

#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    pub city: String,
    pub zip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub address: Address,
}

// --- Lens の定義 ---

pub fn person_address_lens() -> Lens<Person, Address> {
    Lens {
        get: |p| p.address.clone(),
        set: |p, addr| Person { address: addr, ..p },
    }
}

pub fn person_age_lens() -> Lens<Person, u32> {
    Lens {
        get: |p| p.age,
        set: |p, age| Person { age, ..p },
    }
}

pub fn address_city_lens() -> Lens<Address, String> {
    Lens {
        get: |a| a.city.clone(),
        set: |a, city| Address { city, ..a },
    }
}

/// 2つの Lens を合成: S → A → B を S → B にする
pub fn compose<'a, S: Clone, A: Clone, B: Clone>(
    outer: &'a Lens<S, A>,
    inner: &'a Lens<A, B>,
) -> impl Fn(&S) -> B + 'a {
    move |s| inner.view(&outer.view(s))
}

/// 2つの Lens を合成して更新する: S の中の B を更新する
pub fn compose_update<S: Clone, A: Clone, B: Clone>(
    outer: &Lens<S, A>,
    inner: &Lens<A, B>,
    s: S,
    b: B,
) -> S {
    let a = outer.view(&s);
    let new_a = inner.update(a, b);
    outer.update(s, new_a)
}

// --- Prism: enum のバリアントへの条件付きアクセス ---

#[derive(Debug, Clone)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

/// Circle バリアントの radius を取り出す（Prism の preview に相当）
pub fn circle_radius(shape: &Shape) -> Option<f64> {
    match shape {
        Shape::Circle { radius } => Some(*radius),
        _ => None,
    }
}

/// Circle バリアントの radius を更新する（Prism の over に相当）
pub fn update_circle_radius<F>(shape: Shape, f: F) -> Shape
where
    F: FnOnce(f64) -> f64,
{
    match shape {
        Shape::Circle { radius } => Shape::Circle { radius: f(radius) },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_person() -> Person {
        Person {
            name: "Alice".to_string(),
            age: 30,
            address: Address {
                city: "Tokyo".to_string(),
                zip: "100-0001".to_string(),
            },
        }
    }

    #[test]
    fn test_lens_view() {
        let person = sample_person();
        let addr_lens = person_address_lens();
        let city_lens = address_city_lens();

        let address = addr_lens.view(&person);
        let city = city_lens.view(&address);
        assert_eq!(city, "Tokyo");
    }

    #[test]
    fn test_lens_update() {
        let person = sample_person();
        let addr_lens = person_address_lens();
        let city_lens = address_city_lens();

        let new_address = city_lens.update(addr_lens.view(&person), "Osaka".to_string());
        let updated = addr_lens.update(person, new_address);

        assert_eq!(updated.address.city, "Osaka");
        assert_eq!(updated.address.zip, "100-0001"); // zip は変わらない
        assert_eq!(updated.name, "Alice");            // name も変わらない
    }

    #[test]
    fn test_lens_modify() {
        let person = sample_person();
        let age_lens = person_age_lens();

        let updated = age_lens.modify(person, |age| age + 1);
        assert_eq!(updated.age, 31);
    }

    #[test]
    fn test_lens_compose() {
        let person = sample_person();
        let addr_lens = person_address_lens();
        let city_lens = address_city_lens();

        // 合成した get
        let get_city = compose(&addr_lens, &city_lens);
        assert_eq!(get_city(&person), "Tokyo");

        // 合成した update
        let updated = compose_update(&addr_lens, &city_lens, person, "Kyoto".to_string());
        assert_eq!(updated.address.city, "Kyoto");
    }

    #[test]
    fn test_prism_preview() {
        let shapes = vec![
            Shape::Circle { radius: 5.0 },
            Shape::Rectangle { width: 3.0, height: 4.0 },
            Shape::Circle { radius: 2.0 },
        ];

        let radii: Vec<f64> = shapes.iter().filter_map(circle_radius).collect();
        assert_eq!(radii, vec![5.0, 2.0]);
    }

    #[test]
    fn test_prism_update() {
        let circle = Shape::Circle { radius: 3.0 };
        let rect = Shape::Rectangle { width: 2.0, height: 5.0 };

        let doubled_circle = update_circle_radius(circle, |r| r * 2.0);
        let unchanged_rect = update_circle_radius(rect.clone(), |r| r * 2.0);

        assert!(matches!(doubled_circle, Shape::Circle { radius } if radius == 6.0));
        assert!(matches!(unchanged_rect, Shape::Rectangle { width: 2.0, height: 5.0 }));
    }

    #[test]
    fn test_immutability() {
        // 元の値が変わらないことを確認
        let person = sample_person();
        let age_lens = person_age_lens();
        let updated = age_lens.update(person.clone(), 99);

        assert_eq!(person.age, 30);   // 元は変わらない
        assert_eq!(updated.age, 99);  // 新しい値のみ変わる
    }
}

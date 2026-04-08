use basics::greet;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Order {
    id: u32,
    subtotal: i32,
    coupon: Option<i32>,
}

// Pure function: deterministic business rule with no side effects.
fn calculate_final_price(order: &Order) -> i32 {
    let discounted = match order.coupon {
        Some(discount) => order.subtotal - discount,
        None => order.subtotal,
    };
    discounted.max(0)
}

// Effectful function: handles observable actions like printing/saving.
fn checkout(order: &Order, timestamp: u64) {
    let final_price = calculate_final_price(order);
    println!("[{}] checkout start id={}", timestamp, order.id);
    println!("subtotal={}, coupon={:?}", order.subtotal, order.coupon);
    println!("final price={}", final_price);
    println!("(simulate) save order {} with final price {}", order.id, final_price);
}

fn main() {
    let order_a = Order {
        id: 1,
        subtotal: 3_000,
        coupon: Some(500),
    };

    let order_b = Order {
        id: 2,
        subtotal: 400,
        coupon: Some(800),
    };

    // Same input -> same output (referential transparency)
    assert_eq!(calculate_final_price(&order_a), 2_500);
    assert_eq!(calculate_final_price(&order_a), 2_500);
    assert_eq!(calculate_final_price(&order_b), 0);

    // Side effect boundary: obtain timestamp at the shell layer.
    let ts = 1_711_111_111;
    checkout(&order_a, ts);
    checkout(&order_b, ts);

    // Reuse pure function from the library.
    let message = greet("Functional Core", ts);
    println!("{}", message);
}

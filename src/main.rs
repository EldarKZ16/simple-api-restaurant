use std::{sync::Arc, thread};

use simple_api_restaurant::InMemoryOrderRepository;
use simple_api_restaurant::OrderRepository;

fn main() {
    let in_memory_repo = Arc::new(InMemoryOrderRepository::new());
    let table_number = 1;

    let mut handles = vec![];
    for _ in 0..10 {
        let in_memory_repo = Arc::clone(&in_memory_repo);
        let item = "Sushi".to_string();
        let quantity = 1;

        let handle = thread::spawn(move || {
            in_memory_repo.add(table_number, item, quantity);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let orders = in_memory_repo.get_by_table_number(table_number);
    println!("Orders for table {}: {:?}", table_number, orders);
}

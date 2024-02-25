use std::{sync::Arc, thread};

use simple_api_restaurant::InMemoryOrderRepository;
use simple_api_restaurant::OrderService;

fn main() {
    let in_memory_repo = Arc::new(InMemoryOrderRepository::new());
    let order_service = Arc::new(OrderService::new(in_memory_repo));
    let table_number = 1;

    let mut handles = vec![];
    for _ in 0..10 {
        let order_service = Arc::clone(&order_service);
        let item = "Sushi".to_string();
        let quantity = 1;

        let handle = thread::spawn(move || {
            match order_service.add_order(table_number, item, quantity) {
                Ok(_) => println!("Order added"),
                Err(e) => println!("Failed to add order: {:?}", e),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let orders = order_service.get_remaining_orders_by_table_number(table_number);
    println!("Orders for table {}: {:#?}", table_number, orders);
    assert_eq!(orders.unwrap().len(), 10);
}

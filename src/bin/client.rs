use tokio;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let tasks: Vec<_> = (0..10).map(|_| {
        let client = client.clone();

        tokio::spawn(async move {
            let res = client.post("http://127.0.0.1:3030/v1/orders")
                .json(&serde_json::json!({
                    "table_number": 1,
                    "menu_item": "Sushi",
                    "quantity": 2
                }))
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Order added");
                    } else {
                        println!("Failed to add order: {}", response.status());
                    }
                }
                Err(e) => println!("Failed to send request: {}", e),
            }
        })
    }).collect();

    for task in tasks {
        let _ = task.await;
    }

    println!("Finished");
}
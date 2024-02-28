# Simple Restaurant Api

### Prerequisites
- Stable version of Rust. The project was build with version [1.76.0](https://blog.rust-lang.org/2024/02/08/Rust-1.76.0.html)

### How to install and run:
1. Clone the repo
2. Build the project
```
cargo build
```
3. Run the server
```
cargo run --bin simple-api-restaurant
```
4. REST API should be accessible at `localhost:3030`

### Testing:
1. Unit tests
```
cargo test
```
2. Client that calls the REST API endpoints
```
cargo run --bin client
```

### Usage
There are several endpoints that can be called:
```
GET /healthcheck
returns 200 OK if service is running

POST /v1/orders
{
    "table_number": 1,
    "menu_item": "Sushi",
    "quantity": 2
}
returns 200 OK if order is added


GET /v1/orders/{id}
returns 200 OK if order exists:
{
    "id": 1,
    "table_number": 1,
    "menu_item": "Pizza",
    "quantity": 2,
    "time_to_cook": 7 // this number is in minutes
}

returns 404 NOT FOUND if order doesn't exist


GET /v1/orders?table_number={table_number}
returns 200 OK with list of orders (if table doesn't have an order, return empty array []):
[
    {
        "id": 1,
        "table_number": 1,
        "menu_item": "Sushi",
        "quantity": 2,
        "time_to_cook": 1
    }
]

DELETE /v1/orders/{id}
returns 200 OK 
```

## Developer notes

### Overall spent time
- Time spent to complete the task is about ~15 hours 
- Time spent to spend learning [Rust Programming Language](https://doc.rust-lang.org/book/title-page.html) and libraries is about 2-3 days. I spent too much time to understand the basics and syntax of language


### Design consideration
I went with the Repository approach as it can be generic and extendable. It took some time to read about Concurrency and find out about Arc and Mutex as a good approach for this task. The Arc allows multiple threads to own a shared instance. The Mutex guards the HashMap against concurrent access. About HashMap, it was natural to choose, as it is good for data retrieval. Overall, this part of the code was crucial. 

### Limitations/Possible Improvements:
1. `OrderService.get_remaining_order_by_table_number` is very inefficient, cause it retrieves all the orders from hashma and filters based on table_number. It works fine for small set of orders. \
**How to improve?** 
- Create another HashMap that has table number as key and order id as a value. In that case, the retrieval will be quick
- Implement PostgreSQLRepository that uses DB, create an index for the table_number. Refactor OrderService to use PostgreSQLRepository

2. `InMemoryRepository` uses Mutex that blocks the entire thread. Possibly need to replace it with tokio Mutex that uses semantic blocking

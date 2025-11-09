## Scoring
### Basic
Builds successfully. Inputs and outputs are CSV, and the `rust_decimal` crate is used to handle decimal values.

### Completeness
Covers all transaction types. See the comprehensive test suite: [test_comprehensive_csv](https://github.com/lucaspere/payment-engine/blob/c6fa58ac987dcdb94106eb7a3dc926d10402605d/tests/integration_tests.rs#L86).

### Correctness
I added both unit and integration tests that validate the expected use cases, following the recommendations in The [Rust Programming Language — Ch. 11.3 (Test Organization)](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

### Safety and Robustness
Error handling uses the standard `Error` trait and reports errors to the user. I chose this approach to keep error types simple and predictable.

### Efficiency
Efficient for single-threaded processing: the `csv` crate provides a lazy iterator over records, so the implementation does not load the entire file into memory. For concurrent TCP streams, the `PaymentEngine` would need synchronization to allow concurrent mutation, like wrapping shared state in a `Mutex`. In concurrent contexts, `Stream` might be better than `Iterator`, because Stream is designed to work with async operations.

### Maintainability
The codebase is simple and modular. I implemented traits for data sources and sinks (Strategy Pattern), which make it easy to swap providers. Each transaction type has a dedicated function that encapsulates its processing logic.
use tails_native_macros::tails_module;

#[tails_module(name = "my-tails-module")]
mod my_module {
    use tails_native_macros::{tails_class, tails_function};

    // --- Counter class ---

    #[allow(dead_code)]
    pub struct Counter {
        count: f64,
    }

    #[tails_class]
    impl Counter {
        pub fn new(initial: f64) -> Self {
            Counter { count: initial }
        }

        pub fn increment(&mut self) {
            self.count += 1.0;
        }

        pub fn decrement(&mut self) {
            self.count -= 1.0;
        }

        pub fn get_count(&self) -> f64 {
            self.count
        }
    }

    // --- Simple functions ---

    #[tails_function]
    pub fn greet(name: String) -> String {
        format!("Hello, {}!", name)
    }

    #[tails_function]
    pub fn add(a: f64, b: f64) -> f64 {
        a + b
    }

    #[tails_function]
    pub fn multiply(a: f64, b: f64) -> f64 {
        a * b
    }
}

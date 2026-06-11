use std::time::Duration;
use std::thread;

/// Represents a result from a primary or fallback operation
#[derive(Debug)]
enum FallbackResult<T> {
    Primary(T),
    Fallback(T),
    Exhausted(String),
}

/// Configuration for fallback behavior
struct FallbackConfig {
    primary_timeout: Duration,
    fallback_timeout: Duration,
    max_retries: u32,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            primary_timeout: Duration::from_millis(100),
            fallback_timeout: Duration::from_millis(200),
            max_retries: 3,
        }
    }
}

/// A fallback strategy that tries primary, then degrades gracefully
struct FallbackStrategy {
    config: FallbackConfig,
    primary_fail_count: u32,
}

impl FallbackStrategy {
    fn new(config: FallbackConfig) -> Self {
        Self {
            config,
            primary_fail_count: 0,
        }
    }

    fn execute<F, G, T>(&mut self, primary: F, fallback: G) -> FallbackResult<T>
    where
        F: Fn() -> Result<T, String>,
        G: Fn() -> Result<T, String>,
    {
        // Try primary with retries
        for attempt in 0..self.config.max_retries {
            match primary() {
                Ok(val) => {
                    self.primary_fail_count = 0;
                    return FallbackResult::Primary(val);
                }
                Err(_) => {
                    self.primary_fail_count += 1;
                    if attempt < self.config.max_retries - 1 {
                        thread::sleep(self.config.primary_timeout);
                    }
                }
            }
        }

        // Fall back to degraded operation
        match fallback() {
            Ok(val) => FallbackResult::Fallback(val),
            Err(e) => FallbackResult::Exhausted(e),
        }
    }

    fn fail_count(&self) -> u32 {
        self.primary_fail_count
    }
}

fn main() {
    let mut strategy = FallbackStrategy::new(FallbackConfig::default());

    // Simulate a primary that always fails
    let result = strategy.execute(
        || Err("database unavailable".into()),
        || Ok("cached response".to_string()),
    );
    println!("Scenario 1 (primary fails): {:?}", result);

    // Simulate a primary that succeeds
    let result2 = strategy.execute(
        || Ok("live data".to_string()),
        || Ok("cached response".to_string()),
    );
    println!("Scenario 2 (primary succeeds): {:?}", result2);

    // Both fail
    let result3: FallbackResult<String> = strategy.execute(
        || Err("db down".into()),
        || Err("cache empty".into()),
    );
    println!("Scenario 3 (all fail): {:?}", result3);

    println!("Primary fail count: {}", strategy.fail_count());
}

# Fallback Strategy

**Fallback strategy** is a resilience pattern where an operation first attempts a **primary** path (e.g., a live database), and upon repeated failure, degrades gracefully to a **fallback** path (e.g., a cache or stale snapshot) — ensuring the system remains responsive even when dependencies are degraded.

## Why It Matters

Distributed systems fail in unpredictable ways: network partitions, database connection exhaustion, rate limits, and GC pauses. The circuit breaker pattern popularized by Netflix's Hystrix uses fallback strategies to prevent cascading failures. Without fallbacks, a single slow dependency can exhaust thread pools and bring down an entire service. This crate implements a configurable fallback executor with retry budgets, timeout-aware primary attempts, and a clean three-state result type (`Primary`, `Fallback`, `Exhausted`) that lets callers react appropriately.

## How It Works

### Execution Flow

```
execute(primary, fallback):
  for attempt in 0..max_retries:
      result = primary()
      if Ok: reset fail counter → return Primary(value)
      else:  increment fail counter, sleep(primary_timeout)
  
  result = fallback()
  if Ok: return Fallback(value)
  else:  return Exhausted(error)
```

### Retry Budget

The `FallbackConfig` controls three parameters:

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `primary_timeout` | 100ms | Backoff between primary retries |
| `fallback_timeout` | 200ms | Max wait before giving fallback |
| `max_retries` | 3 | Primary attempts before falling back |

### Three-State Result

`FallbackResult<T>` distinguishes outcomes explicitly:
- **`Primary(T)`** — the primary path succeeded; the system is healthy.
- **`Fallback(T)`** — primary failed but fallback provided a valid (possibly stale) result.
- **`Exhausted(String)`** — both paths failed; the error message explains why.

This is superior to `Result<T, E>` because it communicates **which path produced the answer**, enabling observability (metrics on fallback rate) and downstream logic (trigger cache refresh when serving stale data).

### Complexity

- Primary attempts: **O(max_retries)** function calls.
- Fallback: **O(1)** additional call.
- Total latency: O(max_retries × primary_timeout + fallback_timeout) in the worst case.

## Quick Start

```rust
use std::time::Duration;

let mut strategy = FallbackStrategy::new(FallbackConfig::default());

// Scenario 1: Primary fails, fallback serves cached data
let result = strategy.execute(
    || Err("database unavailable".into()),
    || Ok("cached response".to_string()),
);
assert!(matches!(result, FallbackResult::Fallback(_)));

// Scenario 2: Primary succeeds
let result = strategy.execute(
    || Ok("live data".to_string()),
    || Ok("cached".to_string()),
);
assert!(matches!(result, FallbackResult::Primary(_)));

// Scenario 3: Both fail
let result: FallbackResult<String> = strategy.execute(
    || Err("db down".into()),
    || Err("cache empty".into()),
);
assert!(matches!(result, FallbackResult::Exhausted(_)));
```

## API

| Type / Function | Description |
|----------------|-------------|
| `FallbackStrategy` | Executor with retry counting; `execute(primary, fallback) → FallbackResult<T>` |
| `FallbackConfig` | Configuration: `primary_timeout`, `fallback_timeout`, `max_retries` |
| `FallbackResult<T>` | Enum: `Primary(T)`, `Fallback(T)`, `Exhausted(String)` |

## Architecture Notes

Part of the **SuperInstance** reliability framework. Fallback strategies wrap Fleet API calls to ensure the system degrades gracefully under partial failures. This realizes **γ + η = C**: γ (correct error handling) and η (graceful degradation) combine to maintain system availability.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the full system design.

## References

1. Nygard, M. T. *Release It!: Design and Deploy Production-Ready Software*, 2nd ed. Pragmatic Bookshelf, 2018. Chapter 5: Circuit Breaker.
2. Netflix. "Hystrix: Latency and Fault Tolerance for Distributed Systems." <https://github.com/Netflix/Hystrix>.
3. Golubovic, D. "Resilience Patterns: Circuit Breaker, Retry, Fallback." *QCon London 2019*.

## License

MIT

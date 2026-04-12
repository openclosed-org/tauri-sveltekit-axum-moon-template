//! Infrastructure adapters for admin service.
//!
//! These adapters implement the admin ports by delegating to concrete services
//! (tenant-service, counter-service) or direct database access.
//!
//! ## Design
//! - Adapters live in the infrastructure layer
//! - They implement port traits (TenantRepository, CounterRepository)
//! - The application layer depends only on ports, not these adapters
//! - Wiring/injection happens at the server composition root

// Note: Concrete adapters are commented out because they would create
// dependencies on other services, which violates service isolation rules.
//
// In production, these adapters should be implemented as:
// 1. Direct database access (preferred — keeps services independent)
// 2. HTTP client calls to other service endpoints
// 3. Event-based queries via the event bus
//
// For now, we provide stub implementations that demonstrate the structure.
// See servers/api/src/routes/admin.rs for actual composition logic.

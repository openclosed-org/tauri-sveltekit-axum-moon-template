//! {{endpoint-name}} adapter — translates service trait to HTTP response.
//!
//! This adapter:
//! 1. Calls the service trait method
//! 2. Maps domain errors to HTTP status codes
//! 3. Serializes the response

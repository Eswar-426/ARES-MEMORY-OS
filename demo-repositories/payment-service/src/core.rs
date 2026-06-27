/// Core trait abstracting payment processors.
/// Implements REQ-12.
pub trait PaymentProvider {
    fn process_payment(&self, amount: u64, currency: &str) -> Result<(), String>;
    fn refund_payment(&self, transaction_id: &str) -> Result<(), String>;
}

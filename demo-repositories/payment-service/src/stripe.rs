use crate::core::PaymentProvider;

pub struct StripeProvider;

impl PaymentProvider for StripeProvider {
    fn process_payment(&self, amount: u64, currency: &str) -> Result<(), String> {
        println!("Processing {} {} via Stripe", amount, currency);
        Ok(())
    }
    
    fn refund_payment(&self, transaction_id: &str) -> Result<(), String> {
        println!("Refunding {} via Stripe", transaction_id);
        Ok(())
    }
}

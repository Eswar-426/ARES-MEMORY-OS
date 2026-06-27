use crate::core::PaymentProvider;

pub struct PayPalProvider;

impl PaymentProvider for PayPalProvider {
    fn process_payment(&self, amount: u64, currency: &str) -> Result<(), String> {
        println!("Processing {} {} via PayPal", amount, currency);
        Ok(())
    }
    
    fn refund_payment(&self, transaction_id: &str) -> Result<(), String> {
        println!("Refunding {} via PayPal", transaction_id);
        Ok(())
    }
}

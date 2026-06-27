mod core;
mod stripe;
mod paypal;

use core::PaymentProvider;
use stripe::StripeProvider;
use paypal::PayPalProvider;

fn main() {
    let provider: Box<dyn PaymentProvider> = if std::env::var("USE_PAYPAL").is_ok() {
        Box::new(PayPalProvider)
    } else {
        Box::new(StripeProvider)
    };
    
    let _ = provider.process_payment(100, "USD");
}

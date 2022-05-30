//! For querying beefy signed commitment.

use jsonrpsee::core::client::Subscription;
use jsonrpsee::core::DeserializeOwned;
use crate::rpc::SignedCommitment;

/// Beefy justification subscription
pub struct BeefySubscription {
    subscription: Subscription<SignedCommitment>,
}

impl BeefySubscription {
    /// Creates a new beefy justification subscription.
    pub fn new(subscription: Subscription<SignedCommitment>) -> Self {
        Self { subscription }
    }

    /// Gets the next signed commitment.
    pub async fn next(&mut self) -> Option<SignedCommitment> {
        read_subscription_response("BeefySubscription", &mut self.subscription).await
    }
}

async fn read_subscription_response<T>(
    _sub_name: &str,
    sub: &mut Subscription<T>,
) -> Option<T>
where
    T: DeserializeOwned,
{
    match sub.next().await {
        Some(Ok(next)) => Some(next),
        Some(Err(_)) => None,
        None => {
            None
        }
    }
}
pub use crate::domain::SubscriberName;
pub use crate::domain::SubscriberEmail;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

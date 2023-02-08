use rumqttc::{Event, EventLoop};
use tokio::sync::broadcast::Sender;

/// Continually polls the [`rumqttc::EventLoop`] and sends the results to a
/// [`tokio::sync::broadcast::Sender`].
// TODO: Needs proper error handling.
pub async fn poll_event_loop(mut event_loop: EventLoop, sender: Sender<Event>) {
	loop {
		let event = event_loop.poll().await;

		match &event {
			Ok(v) => {
				let _ = sender.send(v.to_owned());
			}
			Err(e) => {
				println!("MQTT error: {e:?}");
			}
		};
	}
}

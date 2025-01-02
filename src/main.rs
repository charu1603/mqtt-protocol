use tokio::{sync::mpsc, task, time};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel::<String>(10);

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("hello/rumqtt", QoS::AtMostOnce).await?;

    task::spawn(async move {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    println!("Publisher received notification: {}", msg);

                    match client.publish("hello/rumqtt", QoS::AtLeastOnce, false, msg).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed to publish message: {:?}", e);
                        }
                    }

                    time::sleep(Duration::from_secs(1)).await;
                }
                None => {
                    eprintln!("Channel closed, stopping publisher.");
                    break;
                }
            }
        }
    });

    let subscriber_tx = tx.clone();
    task::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                
                    println!("Received from MQTT: {:?}", notification);

                    if let Err(e) = subscriber_tx.send("Send next message".to_string()).await {
                        eprintln!("Failed to notify publisher: {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error polling event loop: {:?}", e);
                    break;
                }
            }
        }
    });

    loop {
        time::sleep(Duration::from_secs(5)).await;
    }
}


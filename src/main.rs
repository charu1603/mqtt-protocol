use tokio::{sync::mpsc, task, time};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   
    let (tx, mut rx) = mpsc::channel::<String>(10);

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let ( client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("hello/rumqtt", QoS::AtMostOnce).await?;

    task::spawn(async move {
        loop { 
            if let Some(msg) = rx.recv().await {
                println!("Publisher received notification: {}", msg);
                
                if let Err(e) = client.publish("hello/rumqtt", QoS::AtLeastOnce, false, msg).await {
                    eprintln!("Failed to publish message: {:?}", e);
                }

           
                time::sleep(Duration::from_secs(1)).await;
            } else {
                eprintln!("Channel closed, stopping publisher.");
                break;
            }
        }
    });

   
    let subscriber_tx = tx.clone();
    task::spawn(async move {
        loop {
            if let Some(notification) = eventloop.poll().await.ok() {
                println!("Received from MQTT: {:?}", notification);

                
                if let Err(e) = subscriber_tx.send("Send next message".to_string()).await {
                    eprintln!("Failed to notify publisher: {:?}", e);
                    break;
                }
            } else {
                eprintln!("Error polling event loop.");
                break;
            }
        }
    });

  
    loop {
        time::sleep(Duration::from_secs(5)).await;
    }
}

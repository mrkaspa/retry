use anyhow::Result;
use futures::executor::block_on;
use lapin::options::{
    BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::{AMQPValue, FieldTable};
use lapin::{options::*, BasicProperties, Channel, Connection, ConnectionProperties};
use log::info;

async fn connect(addr: &String) -> Result<Channel> {
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    info!("Connected");
    let channel = conn.create_channel().await?;
    info!("Channel created");
    Ok(channel)
}

async fn setup(ch: &Channel) -> Result<&str> {
    ch.exchange_declare(
        "retries.dlx-ex",
        lapin::ExchangeKind::Direct,
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    )
    .await?;
    ch.exchange_declare(
        "retries.retry-ex",
        lapin::ExchangeKind::Direct,
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    )
    .await?;
    ch.queue_declare(
        "retries.wait-queue",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    )
    .await?;
    ch.queue_bind(
        "retries.wait-queue",
        "retries.dlx-ex",
        "do-retry",
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;
    let mut args = FieldTable::default();
    args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString("retries.dlx-ex".into()),
    );
    args.insert(
        "x-dead-letter-routing-key".into(),
        AMQPValue::LongString("do-retry".into()),
    );
    ch.queue_declare("retries.exec-queue", QueueDeclareOptions::default(), args)
        .await?;
    let mut args = FieldTable::default();
    args.insert("x-message-ttl".into(), 5000.into());
    args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString("retries.retry-ex".into()),
    );
    ch.queue_bind(
        "retries.exec-queue",
        "retries.retry-ex",
        "do-retry",
        QueueBindOptions::default(),
        args,
    )
    .await?;
    Ok("retries.exec-queue")
}

async fn publish(ch: Channel, exchange: &str, routing_key: &str, payload: Vec<u8>) -> Result<()> {
    ch.basic_publish(
        exchange,
        routing_key,
        BasicPublishOptions::default(),
        payload,
        BasicProperties::default(),
    )
    .await?;
    Ok(())
}

async fn amain(addr: &String) -> Result<()> {
    let channel = connect(addr).await?;
    let queue = setup(&channel).await?;
    let consumer = channel
        .basic_consume(
            queue,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!("will consume");
    for delivery in consumer {
        let (channel, delivery) = delivery.expect("error in consumer");
        channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .await
            .expect("ack");
    }
    Ok(())
}

#[allow(dead_code)]
fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let future = amain(&addr);
    block_on(future)?;
    Ok(())
}

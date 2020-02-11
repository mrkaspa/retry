use amq_protocol_types::{AMQPValue, ShortString};
use anyhow::Result;
use futures::executor::block_on;
use futures::{future::FutureExt, stream::StreamExt};
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties, Queue};
use log::{debug, info};

async fn connect(addr: &String) -> Result<Channel> {
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    Ok(channel)
}

async fn setup(ch: &Channel) -> Result<Queue> {
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
    let mut args = FieldTable::default();
    args.insert(
        ShortString::from(String::from("x-dead-letter-exchange")),
        AMQPValue::ShortString(ShortString::from(String::from("retries.dlx-ex"))),
    );
    args.insert(
        ShortString::from(String::from("x-dead-letter-routing-key")),
        AMQPValue::ShortString(ShortString::from(String::from("do-retry"))),
    );
    let exec_queue = ch
        .queue_declare("retries.exec-queue", QueueDeclareOptions::default(), args)
        .wait()?;
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
        ShortString::from(String::from("x-message-ttl")),
        AMQPValue::ShortInt(5000),
    );
    args.insert(
        ShortString::from(String::from("x-dead-letter-exchange")),
        AMQPValue::ShortString(ShortString::from(String::from("retries.retry-ex"))),
    );
    ch.queue_bind(
        "retries.exec-queue",
        "retries.retry-ex",
        "do-retry",
        QueueBindOptions::default(),
        args,
    )
    .await?;
    Ok(exec_queue)
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
        .clone()
        .basic_consume(
            &queue,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    consumer
        .for_each(move |delivery| {
            let delivery = delivery.expect("Couldn't receive delivery from RabbitMQ.");
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .map(|_| ())
        })
        .await;
    Ok(())
}

#[allow(dead_code)]
fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let future = amain(&addr);
    block_on(future)?;
    Ok(())
    //     futures::executor::spawn(connect(&addr).and_then(|channel| {
    //         let queue = setup(&channel).expect("setup failed");
    //         channel
    //             .basic_consume(
    //                 &queue,
    //                 "",
    //                 BasicConsumeOptions::default(),
    //                 FieldTable::default(),
    //             )
    //             .and_then(move |stream| {
    //                 stream.for_each(move |message| {
    //                     println!(
    //                         "consumer got '{}'",
    //                         std::str::from_utf8(&message.data).unwrap()
    //                     );
    //                     channel.basic_ack(message.delivery_tag, false)
    //                 })
    //             })
    //     }))
    //     .wait_future()
    //     .expect("runtime exited with failure");
}

use amq_protocol_types::{AMQPValue, ShortString};
use futures::{Future, Stream};
use lapin::options::{
    BasicConsumeOptions, BasicGetOptions, BasicPublishOptions, ExchangeBindOptions,
    ExchangeDeclareOptions, ExchangeDeleteOptions, ExchangeUnbindOptions, QueueBindOptions,
    QueueDeclareOptions,
};
use lapin_futures::types::FieldTable;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, Queue};
use log::{debug, info};

fn connect(addr: &String) -> impl Future<Item = Channel, Error = lapin_futures::Error> {
    Client::connect(&addr, ConnectionProperties::default())
        .and_then(|client| client.create_channel())
}

fn setup(ch: &Channel) -> Result<Queue, lapin_futures::Error> {
    ch.exchange_declare(
        "retries.dlx-ex",
        "direct",
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    )
    .wait()?;
    ch.exchange_declare(
        "retries.retry-ex",
        "direct",
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    )
    .wait()?;
    ch.queue_declare(
        "retries.wait-queue",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    )
    .wait()?;
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
    .wait()?;

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
    .wait()?;
    Ok(exec_queue)
}

fn publish(
    ch: Channel,
    exchange: &str,
    routing_key: &str,
    payload: Vec<u8>,
) -> impl Future<Item = (), Error = lapin_futures::Error> {
    ch.basic_publish(
        exchange,
        routing_key,
        payload,
        BasicPublishOptions::default(),
        BasicProperties::default(),
    )
}

#[allow(dead_code)]
fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    futures::executor::spawn(connect(&addr).and_then(|channel| {
        let queue = setup(&channel).expect("setup failed");
        channel
            .basic_consume(
                &queue,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .and_then(move |stream| {
                stream.for_each(move |message| {
                    println!(
                        "consumer got '{}'",
                        std::str::from_utf8(&message.data).unwrap()
                    );
                    channel.basic_ack(message.delivery_tag, false)
                })
            })
    }))
    .wait_future()
    .expect("runtime exited with failure");
}

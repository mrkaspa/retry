use amq_protocol_types::{AMQPValue, ShortInt, ShortString};
use futures::{Future, Stream};
use lapin::options::{
    BasicConsumeOptions, BasicGetOptions, BasicPublishOptions, ExchangeBindOptions,
    ExchangeDeclareOptions, ExchangeDeleteOptions, ExchangeUnbindOptions, QueueBindOptions,
    QueueDeclareOptions,
};
use lapin_futures::types::FieldTable;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties};
use log::{debug, info};

fn connect(addr: &String) -> impl Future<Item = Channel, Error = lapin_futures::Error> {
    Client::connect(&addr, ConnectionProperties::default())
        .and_then(|client| client.create_channel())
}

fn setup(ch: &Channel) -> Result<(), lapin_futures::Error> {
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
    ch.queue_declare("retries.exec-queue", QueueDeclareOptions::default(), args)
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
    Ok(())
}

fn publish(ch: Channel) {}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    futures::executor::spawn(
        Client::connect(&addr, ConnectionProperties::default()).and_then(|client| {
            client.create_channel().and_then(|channel| {
                let id = channel.id();
                info!("created channel with id: {}", id);

                channel.queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default()).and_then(move |_| {
                    info!("channel {} declared queue {}", id, "hello");

                    channel.exchange_declare("hello_exchange", "direct", ExchangeDeclareOptions::default(), FieldTable::default()).and_then(move |_| {
                        channel.queue_bind("hello", "hello_exchange", "hello_2", QueueBindOptions::default(), FieldTable::default()).and_then(move |_| {
                            channel.basic_publish(
                                "hello_exchange",
                                "hello_2",
                                b"hello from tokio".to_vec(),
                                BasicPublishOptions::default(),
                                BasicProperties::default().with_user_id("guest".into()).with_reply_to("foobar".into())
                            ).and_then(move |_| {
                                channel.exchange_bind("hello_exchange", "amq.direct", "test_bind", ExchangeBindOptions::default(), FieldTable::default()).and_then(move |_| {
                                    channel.exchange_unbind("hello_exchange", "amq.direct", "test_bind", ExchangeUnbindOptions::default(), FieldTable::default()).and_then(move |_| {
                                        channel.exchange_delete("hello_exchange", ExchangeDeleteOptions::default()).and_then(move |_| {
                                            channel.close(200, "Bye")
                                        })
                                    })
                                })
                            })
                        })
                    })
                })
            }).and_then(move |_| {
                client.create_channel()
            }).and_then(|channel| {
                let id = channel.id();
                info!("created channel with id: {}", id);

                let c = channel.clone();
                channel.queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default()).and_then(move |queue| {
                    info!("channel {} declared queue {:?}", id, queue);

                    let ch = channel.clone();
                    channel.basic_get("hello", BasicGetOptions::default()).and_then(move |message| {
                        info!("got message: {:?}", message);
                        let message = message.unwrap();
                        info!("decoded message: {:?}", std::str::from_utf8(&message.delivery.data).unwrap());
                        channel.basic_ack(message.delivery.delivery_tag, false)
                    }).and_then(move |_| {
                        ch.basic_consume(&queue, "my_consumer", BasicConsumeOptions::default(), FieldTable::default())
                    })
                }).and_then(|stream| {
                    info!("got consumer stream");

                    stream.for_each(move |message| {
                        debug!("got message: {:?}", message);
                        info!("decoded message: {:?}", std::str::from_utf8(&message.data).unwrap());
                        c.basic_ack(message.delivery_tag, false)
                    })
                })
            })
        }).map_err(|err| eprintln!("An error occured: {}", err))
    ).wait_future().expect("runtime exited with failure")
}

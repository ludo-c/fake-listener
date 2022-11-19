#[macro_use]
extern crate log;

mod dbus_trait;

use crate::dbus_trait::ProducerProxyAsync;
use async_std::stream::StreamExt;
use async_std::task;
use std::error::Error;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use zbus::{dbus_interface, ConnectionBuilder, SignalContext};

pub struct AdaptStruct {}

const BUS_NAME_PRODUCER: &str = "ludo_ic.daemon.producer";
const INTERFACE_NAME_PRODUCER: &str = "/ludo_ic/daemon/producer";
const INTERNAL_TIMER: u64 = 1;

struct Greeter {}

#[dbus_interface(name = "ludo_ic.daemon.producer")]
impl Greeter {
    async fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }

    #[dbus_interface(signal)]
    async fn MySignalEvent(
        ctxt: &SignalContext<'_>,
        val1: i32,
        val2: i32,
    ) -> Result<(), zbus::Error>;
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Activate subscriber for crates logs.
    tracing_subscriber::FmtSubscriber::builder()
        .compact()
        // Display source code file paths
        .with_file(false)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(false)
        // Don't display the event's target (module path)
        .with_target(false)
        // Read env variable
        .with_env_filter(EnvFilter::from_default_env())
        // Build the subscriber
        .finish()
        .init();

    // spawn producer task
    task::spawn(async move {
        let greeter = Greeter {};
        let conn = ConnectionBuilder::session()
            .unwrap()
            .name(BUS_NAME_PRODUCER)?
            .serve_at(INTERFACE_NAME_PRODUCER, greeter)
            .unwrap()
            .build()
            .await
            .unwrap();

        let iface = conn
            .object_server()
            .interface::<_, Greeter>(INTERFACE_NAME_PRODUCER)
            .await
            .unwrap();
        let sc = iface.signal_context();

        loop {
            async_std::task::sleep(std::time::Duration::from_millis(INTERNAL_TIMER)).await;
            //println!("unblocked !");
            Greeter::MySignalEvent(sc, 1, 43).await.unwrap();
        }
    });

    let dbus_conn = ConnectionBuilder::session()?.build().await?;

    let dbus_conn_listener = dbus_conn.clone();
    loop {
        let proxy = ProducerProxyAsync::builder(&dbus_conn_listener)
            .cache_properties(zbus::CacheProperties::No)
            .build()
            .await?;
        let mut stream = proxy.receive_my_signal_event().await.unwrap();
        let _ = stream.next().await.unwrap();
        error!("Signal received."); // So it is visible
        drop(proxy);
    }
}

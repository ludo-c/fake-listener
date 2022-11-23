#[macro_use]
extern crate log;

mod dbus_trait;

use crate::dbus_trait::ProducerProxyAsync;
use std::error::Error;
use std::time::Duration;
use tokio::time::timeout;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use zbus::export::futures_util::stream::StreamExt;
use zbus::{dbus_interface, ConnectionBuilder};

const BUS_NAME_ADAPTOR: &str = "ludo_ic.daemon.other";
const INTERFACE_NAME_ADAPTOR: &str = "/ludo_ic/daemon/other";

pub struct AdaptStruct {}

#[dbus_interface(name = "ludo_ic.daemon.other")]
impl AdaptStruct {
    async fn SayHello(&self) {
        info!("Hello");
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
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

    let dbus_adaptor = AdaptStruct {};

    let dbus_conn = ConnectionBuilder::session()?
        .name(BUS_NAME_ADAPTOR)?
        .serve_at(INTERFACE_NAME_ADAPTOR, dbus_adaptor)?
        .build()
        .await?;

    let dbus_conn_listener = dbus_conn.clone();

    let proxy = ProducerProxyAsync::builder(&dbus_conn_listener)
        .cache_properties(zbus::CacheProperties::No)
        .build()
        .await
        .unwrap();
    debug!("proxy ok");

    loop {
        debug!("new loop");
        let mut stream = proxy
            .receive_my_signal_event()
            .await
            .map_err(|e| error!("Cannot listen signal {}", e))
            .unwrap();
        debug!("stream ok");
        if timeout(Duration::from_secs(5), stream.next())
            .await
            .is_err()
        {
            info!("reproduced");
            break;
        };
        error!("Signal received."); // So it is visible
    }

    Ok(())
}

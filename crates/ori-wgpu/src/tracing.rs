use std::{env, error::Error};

use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, EnvFilter, Layer};

use ori_core::tracing;

pub fn init_tracing() -> Result<(), Box<dyn Error>> {
    let mut filter = EnvFilter::default()
        .add_directive("wgpu=warn".parse()?)
        .add_directive("naga=warn".parse()?)
        .add_directive("winit=warn".parse()?)
        .add_directive("mio=warn".parse()?)
        .add_directive(LevelFilter::DEBUG.into());

    if let Ok(env) = env::var(EnvFilter::DEFAULT_ENV) {
        filter = filter.add_directive(env.parse()?);
    }

    let subscriber = tracing_subscriber::registry();

    let fmt_layer = fmt::Layer::default().with_filter(filter);
    let subscriber = subscriber.with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

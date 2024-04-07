use std::{env, error::Error};

use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, EnvFilter};

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

    let subscriber = tracing_subscriber::registry().with(filter);

    #[cfg(not(target_arch = "wasm32"))]
    let subscriber = {
        let fmt_layer = tracing_subscriber::fmt::Layer::default();
        subscriber.with(fmt_layer)
    };

    #[cfg(target_arch = "wasm32")]
    let subscriber = {
        let console_layer = tracing_wasm::WASMLayer::new(Default::default());
        subscriber.with(console_layer)
    };

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

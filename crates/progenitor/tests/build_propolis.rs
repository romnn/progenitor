// Copyright 2022 Oxide Computer Company

//! Compile-time smoke test: the propolis serial-console websocket channel compiles.

// ensure that the websocket channel used for serial console compiles.
mod propolis_client {
    progenitor::generate_api!(
        spec = "../../sample_openapi/propolis-server.json",
        interface = Builder,
        tags = Merged,
    );
}

use propolis_client::Client;

#[expect(
    clippy::unwrap_used,
    reason = "compile-only smoke helper; never executed"
)]
fn _ignore() {
    drop(async {
        let _upgraded: reqwest::Upgraded = Client::new("")
            .instance_serial()
            .send()
            .await
            .unwrap()
            .into_inner();
    });
}

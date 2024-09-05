use std::error::Error;

use ori::prelude::*;

const IMAGE_URL: &str = "https://www.rust-lang.org/logos/rust-logo-512x512.png";

async fn load_image(url: &str) -> Result<Image, Box<dyn Error>> {
    let image = reqwest::get(url).await?.bytes().await?;
    Ok(Image::try_load_data(image.to_vec())?)
}

fn ui() -> impl View {
    with_state(
        || String::from(IMAGE_URL),
        |_, state| {
            let url = text_input().text(state.clone()).on_submit(
                |cx, (_, state): &mut (_, String), text| {
                    *state = text;
                    cx.rebuild();
                },
            );
            let url = container(pad(8.0, min_width(400.0, url)));

            let image = memo(
                |(_, state): &mut (_, String)| state.clone(),
                |(_, state): &mut (_, String)| {
                    suspense(text!("Loading..."), {
                        let url = state.clone();

                        async move {
                            match load_image(&url).await {
                                Ok(image) => Ok(image),
                                Err(err) => Err(text!("Error: {}", err)),
                            }
                        }
                    })
                },
            );

            center(vstack![url, image])
        },
    )
}

#[tokio::main]
async fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Suspense Example");
    ori::run_simple(window, ui).unwrap();
}

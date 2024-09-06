use std::error::Error;

use ori::prelude::*;

const IMAGE_URL: &str = "https://www.rust-lang.org/logos/rust-logo-512x512.png";

async fn load_image(url: &str) -> Result<Image, Box<dyn Error>> {
    let image = reqwest::get(url).await?.bytes().await?;
    Ok(Image::try_load_data(image.to_vec())?)
}

fn ui() -> impl View {
    with_data(
        || String::from(IMAGE_URL),
        |url| {
            let input = text_input().text(url.clone());
            let input = input.on_submit(|cx, url, text| {
                *url = text;
                cx.rebuild();
            });

            let input = container(pad(8.0, min_width(400.0, input)));

            let image = memo(
                |url| String::clone(url),
                |url| {
                    let url = url.clone();

                    suspense(async move {
                        match load_image(&url).await {
                            Ok(image) => Ok(image),
                            Err(err) => Err(text!("Error: {}", err)),
                        }
                    })
                    .fallback(text!("Loading..."))
                },
            );

            center(vstack![input, image])
        },
    )
}

#[tokio::main]
async fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Suspense Example");
    ori::run_simple(window, ui).unwrap();
}

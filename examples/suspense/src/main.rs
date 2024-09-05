use ori::prelude::*;

const IMAGE_URL: &str = "https://www.rust-lang.org/logos/rust-logo-512x512.png";

fn ui() -> impl View {
    center(suspense(text!("Loading..."), async {
        let image = reqwest::get(IMAGE_URL)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        Image::load_data(image.to_vec())
    }))
}

#[tokio::main]
async fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Suspense Example");
    ori::run_simple(window, ui).unwrap();
}

pub use winit::platform::android::activity::AndroidApp;

#[doc(hidden)]
pub static ANDROID_APP: std::sync::OnceLock<AndroidApp> = std::sync::OnceLock::new();

#[doc(hidden)]
pub fn set_android_app(app: AndroidApp) {
    ANDROID_APP.set(app).unwrap();
}

#[doc(hidden)]
pub fn get_android_app() -> AndroidApp {
    ANDROID_APP.get().unwrap().clone()
}

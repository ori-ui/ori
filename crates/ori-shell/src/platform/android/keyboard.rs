use android_activity::AndroidApp;
use jni::{
    objects::{JObject, JValue},
    JavaVM,
};
use tracing::error;

pub fn show_soft_input(app: &AndroidApp, show: bool) {
    let vm = match unsafe { JavaVM::from_raw(app.vm_as_ptr() as _) } {
        Ok(vm) => vm,
        Err(err) => {
            error!("Failed to get JavaVM: {:?}", err);
            return;
        }
    };

    let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
    let mut env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(err) => {
            error!("Failed to attach current thread: {:?}", err);
            return;
        }
    };

    let class_cx = match env.find_class("android/content/Context") {
        Ok(class) => class,
        Err(err) => {
            error!("Failed to find class: {:?}", err);
            return;
        }
    };

    let ims = match env.get_static_field(class_cx, "INPUT_METHOD_SERVICE", "Ljava/lang/String;") {
        Ok(ims) => ims,
        Err(err) => {
            error!("Failed to get field: {:?}", err);
            return;
        }
    };

    let im_manager = match env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[ims.borrow()],
        )
        .unwrap()
        .l()
    {
        Ok(im_manager) => im_manager,
        Err(err) => {
            error!("Failed to get input manager: {:?}", err);
            return;
        }
    };

    let jni_window = match env
        .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
    {
        Ok(jni_window) => jni_window,
        Err(err) => {
            error!("Failed to get window: {:?}", err);
            return;
        }
    };

    let wic = env
        .call_method(
            jni_window,
            "getInsetsController",
            "()Landroid/view/WindowInsetsController;",
            &[],
        )
        .unwrap()
        .l()
        .unwrap();

    let wit = env.find_class("android/view/WindowInsets$Type").unwrap();
    let ime = env
        .call_static_method(&wit, "ime", "()I", &[])
        .unwrap()
        .i()
        .unwrap();

    if show {
        env.call_method(&wic, "show", "(I)V", &[ime.into()])
            .unwrap()
            .v()
            .unwrap();
    } else {
        env.call_method(&wic, "hide", "(I)V", &[ime.into()])
            .unwrap()
            .v()
            .unwrap();
    }
}

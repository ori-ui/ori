use android_activity::AndroidApp;
use jni::{
    objects::{JObject, JString, JValue},
    JavaVM,
};
use ori_core::clipboard::ClipboardBackend;
use tracing::error;

pub struct AndroidClipboard {
    pub app: AndroidApp,
}

impl ClipboardBackend for AndroidClipboard {
    fn get_text(&mut self) -> String {
        get_clipboard(&self.app).unwrap_or_default()
    }

    fn set_text(&mut self, text: &str) {
        set_clipboard(&self.app, text);
    }
}

fn get_clipboard(app: &AndroidApp) -> Option<String> {
    let vm = match unsafe { JavaVM::from_raw(app.vm_as_ptr() as _) } {
        Ok(vm) => vm,
        Err(err) => {
            error!("Failed to get JavaVM: {:?}", err);
            return None;
        }
    };

    let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
    let mut env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(err) => {
            error!("Failed to attach current thread: {:?}", err);
            return None;
        }
    };

    let class_cx = match env.find_class("android/content/Context") {
        Ok(class) => class,
        Err(err) => {
            error!("Failed to find class: {:?}", err);
            return None;
        }
    };

    let clipboard_service =
        match env.get_static_field(class_cx, "CLIPBOARD_SERVICE", "Ljava/lang/String;") {
            Ok(clipboard_service) => clipboard_service,
            Err(err) => {
                error!("Failed to get field: {:?}", err);
                return None;
            }
        };

    let clipboard_manager = match env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[clipboard_service.borrow()],
        )
        .unwrap()
        .l()
    {
        Ok(clipboard_manager) => clipboard_manager,
        Err(err) => {
            error!("Failed to get clipboard manager: {:?}", err);
            return None;
        }
    };

    let clip_data = match env
        .call_method(
            &clipboard_manager,
            "getPrimaryClip",
            "()Landroid/content/ClipData;",
            &[],
        )
        .unwrap()
        .l()
    {
        Ok(clip_data) => clip_data,
        Err(err) => {
            error!("Failed to get clip data: {:?}", err);
            return None;
        }
    };

    if clip_data.is_null() {
        return None;
    }

    let item_count = match env
        .call_method(&clip_data, "getItemCount", "()I", &[])
        .unwrap()
        .i()
    {
        Ok(item_count) => item_count,
        Err(err) => {
            error!("Failed to get item count: {:?}", err);
            return None;
        }
    };

    if item_count == 0 {
        return None;
    }

    let clip_item = match env
        .call_method(
            &clip_data,
            "getItemAt",
            "(I)Landroid/content/ClipData$Item;",
            &[0i32.into()],
        )
        .unwrap()
        .l()
    {
        Ok(clip_item) => clip_item,
        Err(err) => {
            error!("Failed to get clip item: {:?}", err);
            return None;
        }
    };

    let clip_text = match env
        .call_method(&clip_item, "getText", "()Ljava/lang/CharSequence;", &[])
        .unwrap()
        .l()
    {
        Ok(clip_text) => clip_text,
        Err(err) => {
            error!("Failed to get clip text: {:?}", err);
            return None;
        }
    };

    let clip_string = match env
        .call_method(&clip_text, "toString", "()Ljava/lang/String;", &[])
        .unwrap()
        .l()
    {
        Ok(clip_string) => clip_string,
        Err(err) => {
            error!("Failed to get clip string: {:?}", err);
            return None;
        }
    };

    let clip_string = JString::from(clip_string);

    let clip_string = match env.get_string(&clip_string) {
        Ok(clip_string) => clip_string,
        Err(err) => {
            error!("Failed to get clip string: {:?}", err);
            return None;
        }
    };

    clip_string.to_str().ok().map(String::from)
}

fn set_clipboard(app: &AndroidApp, text: &str) {
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

    let clipboard_service =
        match env.get_static_field(class_cx, "CLIPBOARD_SERVICE", "Ljava/lang/String;") {
            Ok(clipboard_service) => clipboard_service,
            Err(err) => {
                error!("Failed to get field: {:?}", err);
                return;
            }
        };

    let clipboard_manager = match env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[clipboard_service.borrow()],
        )
        .unwrap()
        .l()
    {
        Ok(clipboard_manager) => clipboard_manager,
        Err(err) => {
            error!("Failed to get clipboard manager: {:?}", err);
            return;
        }
    };

    let clip_string = env.new_string(text).unwrap();
    let clip_string = JObject::from(clip_string);

    let clip_data = match env
        .call_static_method(
            "android/content/ClipData",
            "newPlainText",
            "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;",
            &[JValue::Object(&clip_string), JValue::Object(&clip_string)],
        )
        .unwrap()
        .l()
    {
        Ok(clip_data) => clip_data,
        Err(err) => {
            error!("Failed to create clip data: {:?}", err);
            return;
        }
    };

    match env.call_method(
        &clipboard_manager,
        "setPrimaryClip",
        "(Landroid/content/ClipData;)V",
        &[JValue::Object(&clip_data)],
    ) {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to set primary clip: {:?}", err);
        }
    }
}

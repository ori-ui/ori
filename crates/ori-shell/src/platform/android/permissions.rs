use android_activity::AndroidApp;
use jni::{
    objects::{JObject, JValue},
    JavaVM,
};
use tracing::error;

pub fn request_permissions(app: &AndroidApp, permissions: &[&str]) {
    if permissions.is_empty() {
        return;
    }

    let vm = match unsafe { JavaVM::from_raw(app.vm_as_ptr() as _) } {
        Ok(vm) => vm,
        Err(e) => {
            error!("Failed to get JavaVM: {:?}", e);
            return;
        }
    };

    let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
    let mut env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(e) => {
            error!("Failed to attach current thread: {:?}", e);
            return;
        }
    };

    let permissions = permissions
        .iter()
        .map(|&permission| env.new_string(permission).unwrap())
        .collect::<Vec<_>>();

    let array = env
        .new_object_array(
            (permissions.len() as i32).into(),
            "java/lang/String",
            &permissions[0],
        )
        .unwrap();

    for (i, permission) in permissions.iter().enumerate() {
        env.set_object_array_element(&array, i as i32, permission)
            .unwrap();
    }

    env.call_method(
        activity,
        "requestPermissions",
        "([Ljava/lang/String;I)V",
        &[JValue::Object(&JObject::from(array)), JValue::from(20)],
    )
    .unwrap()
    .v()
    .unwrap();
}

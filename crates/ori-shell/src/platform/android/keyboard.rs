use std::sync::OnceLock;

use android_activity::AndroidApp;
use crossbeam_channel::{Receiver, Sender};
use jni::{
    objects::{JClass, JObject, JString, JValue},
    sys::jobject,
    JNIEnv, JavaVM,
};
use ori_core::event::Ime;

use super::AndroidError;

static TEXT_COMMITS: OnceLock<Sender<String>> = OnceLock::new();

pub struct ImeState {
    receiver: Receiver<String>,
}

impl Default for ImeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ImeState {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        TEXT_COMMITS.set(sender).unwrap();

        Self { receiver }
    }

    pub fn show(&self, app: &AndroidApp) -> Result<(), AndroidError> {
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as _)? };
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
        let mut env = vm.attach_current_thread()?;

        env.call_method(&activity, "showIME", "()V", &[])?;

        Ok(())
    }

    pub fn hide(&self, app: &AndroidApp) -> Result<(), AndroidError> {
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as _)? };
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
        let mut env = vm.attach_current_thread()?;

        env.call_method(&activity, "hideIME", "()V", &[])?;

        Ok(())
    }

    pub fn set(&mut self, app: &AndroidApp, ime: Ime) -> Result<(), AndroidError> {
        fn cursor_index(text: &str, index: usize) -> usize {
            let mut cursor = 0;

            for (i, c) in text.chars().enumerate() {
                if cursor >= index {
                    return i;
                }

                cursor += c.len_utf8();
            }

            text.len()
        }

        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as _)? };
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
        let mut env = vm.attach_current_thread()?;

        let start = cursor_index(&ime.text, ime.selection.start);
        let end = cursor_index(&ime.text, ime.selection.end);

        let text = env.new_string(ime.text)?;

        env.call_method(
            &activity,
            "setIMEText",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&text)],
        )?;

        env.call_method(
            &activity,
            "setIMESelection",
            "(II)V",
            &[JValue::Int(start as i32), JValue::Int(end as i32)],
        )?;

        Ok(())
    }

    pub fn next_commit(&mut self) -> Option<String> {
        self.receiver.try_recv().ok()
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_ori_oriactivity_OriEditText_nativeCommitText<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    text: JString<'local>,
    new_cursor_position: jni::sys::jint,
) {
    let text: String = env.get_string(&text).unwrap().into();
    tracing::info!("IME committed text: {:?}", text);
    TEXT_COMMITS.get().unwrap().send(text).unwrap();
}

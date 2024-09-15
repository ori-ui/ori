use std::sync::OnceLock;

use android_activity::AndroidApp;
use crossbeam_channel::{Receiver, Sender};
use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv, JavaVM,
};
use ori_core::event::{Capitalize, Ime};

use super::AndroidError;

pub enum ImeEvent {
    CommitText(String),
    DeleteSurroundingText(usize, usize),
}

static IME_EVENTS: OnceLock<Sender<ImeEvent>> = OnceLock::new();

pub struct ImeState {
    receiver: Receiver<ImeEvent>,
}

impl Default for ImeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ImeState {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        IME_EVENTS.set(sender).unwrap();

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
        // android expects the cursor position to be the number of characters from the start of the
        // string, not the number of bytes
        fn cursor_index(text: &str, index: usize) -> usize {
            let mut cursor = 0;
            let mut i = 0;

            for c in text.chars() {
                if cursor >= index {
                    return i;
                }

                cursor += c.len_utf8();
                i += 1;
            }

            i
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

        env.call_method(
            &activity,
            "setIMEMultiline",
            "(Z)V",
            &[JValue::Bool(ime.multiline as u8)],
        )?;

        let text_flaged = match ime.capitalize {
            Capitalize::None => 0,
            Capitalize::Words => env
                .get_static_field("android/text/InputType", "TYPE_TEXT_FLAG_CAP_WORDS", "I")?
                .i()?,
            Capitalize::Sentences => env
                .get_static_field(
                    "android/text/InputType",
                    "TYPE_TEXT_FLAG_CAP_SENTENCES",
                    "I",
                )?
                .i()?,
            Capitalize::All => env
                .get_static_field(
                    "android/text/InputType",
                    "TYPE_TEXT_FLAG_CAP_CHARACTERS",
                    "I",
                )?
                .i()?,
        };

        let input_type = env
            .get_static_field("android/text/InputType", "TYPE_CLASS_TEXT", "I")?
            .i()?;

        env.call_method(
            &activity,
            "setIMEInputType",
            "(I)V",
            &[JValue::Int(input_type | text_flaged)],
        )?;

        Ok(())
    }

    pub fn next_event(&mut self) -> Option<ImeEvent> {
        self.receiver.try_recv().ok()
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_ori_oriactivity_OriEditText_nativeCommitText<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    text: JString<'local>,
    _new_cursor_position: jni::sys::jint,
) {
    let text: String = env.get_string(&text).unwrap().into();
    let event = ImeEvent::CommitText(text);
    IME_EVENTS.get().unwrap().send(event).unwrap();
}

#[no_mangle]
pub unsafe extern "system" fn Java_ori_oriactivity_OriEditText_nativeSetComposingText<'local>(
    _env: JNIEnv<'local>,
    _: JClass<'local>,
    _text: JString<'local>,
    _new_cursor_position: jni::sys::jint,
) {
    // TODO
}

#[no_mangle]
pub unsafe extern "system" fn Java_ori_oriactivity_OriEditText_nativeDeleteSurroundingText<
    'local,
>(
    _env: JNIEnv<'local>,
    _: JClass<'local>,
    before_length: jni::sys::jint,
    after_length: jni::sys::jint,
) {
    let event = ImeEvent::DeleteSurroundingText(before_length as usize, after_length as usize);
    IME_EVENTS.get().unwrap().send(event).unwrap();
}

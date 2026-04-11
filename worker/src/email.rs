use futures_util::TryStreamExt;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::ReadableStream;
use worker_sys::EmailMessage as EmailMessageSys;
use worker_sys::SendEmail as SendEmailSys;

use crate::EnvBinding;
use crate::{send::SendFuture, ByteStream, Headers, Result};

#[derive(Debug)]
pub struct EmailMessage {
    pub(crate) inner: EmailMessageSys,
}

impl EmailMessage {
    /// construct a new email message
    pub fn new(from: &str, to: &str, raw: &str) -> Result<Self> {
        Ok(EmailMessage {
            inner: EmailMessageSys::new(from, to, raw)?,
        })
    }

    /// construct a new email message for a ReadableStream
    pub fn new_from_stream(from: &str, to: &str, raw: &ReadableStream) -> Result<Self> {
        Ok(EmailMessage {
            inner: EmailMessageSys::new_from_stream(from, to, raw)?,
        })
    }

    /// the from field of the email message
    pub fn from_email(&self) -> String {
        self.inner.from().unwrap().into()
    }

    /// the to field of the email message
    pub fn to_email(&self) -> String {
        self.inner.to().unwrap().into()
    }

    /// the headers field of the email message
    pub fn headers(&self) -> Headers {
        Headers(self.inner.headers().unwrap())
    }

    /// the raw email message
    pub fn raw(&self) -> Result<ByteStream> {
        self.inner.raw().map_err(Into::into).map(|rs| ByteStream {
            inner: wasm_streams::ReadableStream::from_raw(rs).into_stream(),
        })
    }

    pub async fn raw_bytes(&self) -> Result<Vec<u8>> {
        self.raw()?
            .try_fold(Vec::new(), |mut bytes, mut chunk| async move {
                bytes.append(&mut chunk);
                Ok(bytes)
            })
            .await
    }

    /// the raw size of the message
    pub fn raw_size(&self) -> u64 {
        self.inner.raw_size().unwrap().value_of() as u64
    }

    /// reject message with reason
    pub fn reject(&self, reason: String) {
        self.inner.set_reject(reason.into()).unwrap()
    }

    /// forward message to recipient
    pub async fn forward(&self, recipient: String, headers: Option<Headers>) -> Result<()> {
        let promise = self.inner.forward(recipient.into(), headers.map(|h| h.0))?;

        let fut = SendFuture::new(JsFuture::from(promise));
        fut.await?;
        Ok(())
    }

    /// reply with email message to recipient
    pub async fn reply(&self, message: EmailMessage) -> Result<()> {
        let promise = self.inner.reply(message.inner)?;

        let fut = SendFuture::new(JsFuture::from(promise));
        fut.await?;
        Ok(())
    }
}

impl From<EmailMessageSys> for EmailMessage {
    fn from(inner: EmailMessageSys) -> Self {
        Self { inner }
    }
}

/// An instance of the send email binding.
#[derive(Debug, Clone)]
pub struct SendEmail {
    pub(crate) inner: SendEmailSys,
}

impl SendEmail {
    /// send an email message
    pub async fn send(&self, message: EmailMessage) -> Result<()> {
        let promise = self.inner.send(message.inner)?;

        let fut = SendFuture::new(JsFuture::from(promise));
        fut.await?;
        Ok(())
    }
}

impl EnvBinding for SendEmail {
    const TYPE_NAME: &'static str = "SendEmail";
}

impl JsCast for SendEmail {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<SendEmailSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self { inner: val.into() }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<SendEmail> for JsValue {
    fn from(send_email: SendEmail) -> Self {
        JsValue::from(send_email.inner)
    }
}

impl AsRef<JsValue> for SendEmail {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

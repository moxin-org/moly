use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::{define_class, msg_send, AllocAnyThread, ClassType, DefinedClass, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};

use crate::capture::{CaptureHandler, Error, Event, Source};

pub fn register_handler<T>(handler: Arc<Mutex<T>>) -> Result<(), Error>
where
    T: CaptureHandler,
{
    let mtm: MainThreadMarker = MainThreadMarker::new()
        .expect("macOS capture support must be initialized on the main thread");

    let application = NSApplication::sharedApplication(mtm);

    let service_provider = ServiceProvider::new(handler);
    unsafe {
        application.setServicesProvider(Some(service_provider.as_super()));
    }

    Ok(())
}

struct ServiceProviderIvars {
    handler: Arc<Mutex<dyn CaptureHandler>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "MolyCaptureServiceProvider"]
    #[ivars = ServiceProviderIvars]
    struct ServiceProvider;

    unsafe impl NSObjectProtocol for ServiceProvider {}

    impl ServiceProvider {
        #[unsafe(method(capture:userData:error:))]
        fn capture(
            &self,
            pasteboard: &NSPasteboard,
            _user_data: Option<&NSString>,
            _error: *mut *mut NSString,
        ) {
            let contents;
            unsafe {
                contents = pasteboard.stringForType(NSPasteboardTypeString);
            }

            if let Some(contents) = contents {
                if let Ok(handler) = self.ivars().handler.lock() {
                    handler.capture(Event {
                        contents: contents.to_string(),
                        source: Source::System,
                    });
                }
            }
        }
    }
);

impl ServiceProvider {
    fn new(handler: Arc<Mutex<dyn CaptureHandler>>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ServiceProviderIvars { handler });
        unsafe { msg_send![super(this), init] }
    }
}

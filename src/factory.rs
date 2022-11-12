use std::{
    mem::transmute,
    sync::atomic::{AtomicU32, Ordering},
};

use windows::{
    core::{IUnknown, Interface, Result, GUID},
    Win32::{
        Foundation::{BOOL, E_NOINTERFACE},
        System::Com::{IClassFactory, IClassFactory_Impl},
    },
};

use crate::RustApo;

pub const LOCK_COUNT: AtomicU32 = AtomicU32::new(0);

#[windows::core::implement(IClassFactory)]
pub struct ClassFactory {}

impl ClassFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl IClassFactory_Impl for ClassFactory {
    fn CreateInstance(
        &self,
        punkouter: &Option<IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        if riid.is_null() {
            return E_NOINTERFACE.ok();
        }

        if !punkouter.is_none() && unsafe { *riid } != IUnknown::IID {
            return E_NOINTERFACE.ok();
        }

        let apo: IUnknown = RustApo::new().into();
        unsafe {
            *ppvobject = transmute(apo);
        }
        Ok(())
    }

    fn LockServer(&self, flock: BOOL) -> Result<()> {
        if flock.as_bool() {
            LOCK_COUNT.fetch_add(1, Ordering::SeqCst);
        } else {
            LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
        }

        Ok(())
    }
}

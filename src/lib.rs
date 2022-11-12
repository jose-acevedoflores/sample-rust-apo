//!
//! TO INSTALL:
//! - regsvr32 < path to apo >
//!
//! Some links:
//! <https://matthewvaneerde.wordpress.com/2010/06/03/how-to-enumerate-wasapi-audio-processing-objects-apos-on-your-system/>
//! Support WinRT <https://github.com/microsoft/windows-rs/issues/1094>
//! Sample return Class <https://github.com/microsoft/windows-rs/blob/1df6f66312ea7cb1e5e08eb7f9f0522b364cc267/crates/tests/component/src/lib.rs>
//! EqualizerAPO dev docs: <https://sourceforge.net/p/equalizerapo/wiki/Developer%20documentation/>

// APO location: HKEY_LOCAL_MACHINE\SOFTWARE\Classes\CLSID\{0129658B-8ED4-47E7-BFA5-E2933B128767}
// APO registration: HKEY_LOCAL_MACHINE\SOFTWARE\Classes\AudioEngine\AudioProcessingObjects\{0129658B-8ED4-47E7-BFA5-E2933B128767}
// Endpoint registration: HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\{5a399dab-d1bb-4caa-9a32-d2cf15dee885}
#![allow(non_snake_case)]
use std::{ffi::c_void, mem::transmute, sync::Mutex};

use factory::ClassFactory;
use once_cell::sync::Lazy;
use widestring::U16CString;
use windows::{
    core::{IUnknown, IUnknown_Vtbl, Interface, Result, GUID, HRESULT, HSTRING, PCWSTR},
    Win32::{
        Foundation::{BOOL, CLASS_E_CLASSNOTAVAILABLE, HINSTANCE, S_FALSE, S_OK},
        Media::Audio::Apo::{
            IAudioMediaType, IAudioProcessingObject, IAudioProcessingObjectConfiguration,
            IAudioProcessingObjectConfiguration_Impl, IAudioProcessingObjectRT,
            IAudioProcessingObjectRT_Impl, IAudioProcessingObject_Impl, IAudioSystemEffects,
            IAudioSystemEffects_Impl, APO_CONNECTION_DESCRIPTOR, APO_CONNECTION_PROPERTY, APO_FLAG,
            APO_FLAG_BITSPERSAMPLE_MUST_MATCH, APO_FLAG_FRAMESPERSECOND_MUST_MATCH,
            APO_FLAG_INPLACE, APO_FLAG_SAMPLESPERFRAME_MUST_MATCH, APO_REG_PROPERTIES,
            FNAPONOTIFICATIONCALLBACK,
        },
        System::{
            Com::IClassFactory,
            LibraryLoader::GetModuleFileNameW,
            Registry::{
                RegCloseKey, RegCreateKeyExW, RegDeleteKeyExW, RegSetValueExW, HKEY,
                HKEY_LOCAL_MACHINE, KEY_SET_VALUE, KEY_WOW64_64KEY, REG_OPEN_CREATE_OPTIONS,
                REG_SZ,
            },
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
};

mod factory;
// link to `audiobaseprocessingobject.lib` in `build.rs`
extern "system" {
    fn RegisterAPO(p: *const APO_REG_PROPERTIES) -> HRESULT;
    fn UnregisterAPO(p: *const GUID) -> HRESULT;
    fn EnumerateAPOs(pfnCallback: FNAPONOTIFICATIONCALLBACK, count: *mut c_void) -> HRESULT;
}

const INST_COUNT: Mutex<u32> = Mutex::new(0);
static CUR_HINSTANCE: Mutex<HINSTANCE> = Mutex::new(HINSTANCE(0));

const FRIENDLY_NAME: Lazy<Vec<u16>> =
    Lazy::new(|| U16CString::from_str("RustAPO").unwrap().into_vec_with_nul());

const APO_PROPS: Lazy<*mut APO_REG_PROPERTIES> = Lazy::new(|| {
    let mut name = [0u16; 256];

    for (idx, char) in FRIENDLY_NAME.iter().enumerate() {
        name[idx] = *char;
    }

    let cr = [0u16; 256];
    Box::into_raw(Box::new(APO_REG_PROPERTIES {
        clsid: IRustAPO::IID,
        Flags: APO_FLAG(
            APO_FLAG_BITSPERSAMPLE_MUST_MATCH.0
                | APO_FLAG_FRAMESPERSECOND_MUST_MATCH.0
                | APO_FLAG_INPLACE.0
                | APO_FLAG_SAMPLESPERFRAME_MUST_MATCH.0,
        ),
        szFriendlyName: name,
        szCopyrightInfo: cr,
        u32MajorVersion: 1,
        u32MinorVersion: 0,
        u32MinInputConnections: 1,
        u32MaxInputConnections: 1,
        u32MinOutputConnections: 1,
        u32MaxOutputConnections: 1,
        u32MaxInstances: u32::MAX,
        u32NumAPOInterfaces: 1,
        iidAPOInterfaceList: [IAudioProcessingObject::IID],
    }))
});

#[no_mangle]
pub extern "system" fn DllMain(ins: HINSTANCE, reason: u32, _res: *const c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        *CUR_HINSTANCE.lock().unwrap() = ins;
    }
    BOOL::from(true)
}

#[windows::core::interface("0129658B-8ED4-47E7-BFA5-E2933B128767")]
unsafe trait IRustAPO: IUnknown {
    unsafe fn MyFunction(&self, out: *mut u32) -> HRESULT;
}

#[windows::core::implement(
    IRustAPO,
    IAudioProcessingObject,
    IAudioSystemEffects,
    IAudioProcessingObjectConfiguration,
    IAudioProcessingObjectRT
)]
pub struct RustApo {
    my_val: u32,
}

impl RustApo {
    pub fn new() -> Self {
        Self { my_val: 5 }
    }
}

impl IRustAPO_Impl for RustApo {
    unsafe fn MyFunction(&self, out: *mut u32) -> HRESULT {
        *out = self.my_val;
        S_OK
    }
}

impl IAudioProcessingObjectRT_Impl for RustApo {
    fn APOProcess(
        &self,
        _u32numinputconnections: u32,
        _ppinputconnections: *const *const APO_CONNECTION_PROPERTY,
        _u32numoutputconnections: u32,
        _ppoutputconnections: *mut *mut APO_CONNECTION_PROPERTY,
    ) {
    }

    fn CalcInputFrames(&self, u32outputframecount: u32) -> u32 {
        u32outputframecount
    }

    fn CalcOutputFrames(&self, u32inputframecount: u32) -> u32 {
        u32inputframecount
    }
}

impl IAudioProcessingObjectConfiguration_Impl for RustApo {
    fn LockForProcess(
        &self,
        _u32numinputconnections: u32,
        _ppinputconnections: *const *const APO_CONNECTION_DESCRIPTOR,
        _u32numoutputconnections: u32,
        _ppoutputconnections: *const *const APO_CONNECTION_DESCRIPTOR,
    ) -> Result<()> {
        Ok(())
    }

    fn UnlockForProcess(&self) -> Result<()> {
        Ok(())
    }
}

impl IAudioSystemEffects_Impl for RustApo {}

impl IAudioProcessingObject_Impl for RustApo {
    fn Reset(&self) -> Result<()> {
        Ok(())
    }

    fn GetLatency(&self) -> Result<i64> {
        Ok(0)
    }

    fn GetRegistrationProperties(&self) -> Result<*mut APO_REG_PROPERTIES> {
        Ok(*APO_PROPS)
    }

    fn Initialize(&self, _cbdatasize: u32, _pbydata: *const u8) -> Result<()> {
        Ok(())
    }

    fn IsInputFormatSupported(
        &self,
        _poppositeformat: &Option<IAudioMediaType>,
        _prequestedinputformat: &Option<IAudioMediaType>,
    ) -> Result<IAudioMediaType> {
        todo!()
    }

    fn IsOutputFormatSupported(
        &self,
        _poppositeformat: &Option<IAudioMediaType>,
        _prequestedoutputformat: &Option<IAudioMediaType>,
    ) -> Result<IAudioMediaType> {
        todo!()
    }

    fn GetInputChannelCount(&self) -> Result<u32> {
        todo!()
    }
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if let Ok(lock) = INST_COUNT.lock() {
        if *lock == 0 {
            S_OK
        } else {
            S_FALSE
        }
    } else {
        S_FALSE
    }
}

#[no_mangle]
pub extern "system" fn DllGetClassObject(
    clsid: *const GUID,
    _iid: *const GUID,
    ppv: *mut *const c_void,
) -> HRESULT {
    if unsafe { *clsid } != IRustAPO::IID {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    let cf: IClassFactory = ClassFactory::new().into();
    unsafe {
        *ppv = transmute(cf);
    }

    S_OK
}

fn get_dll_path() -> U16CString {
    let mut path = [0u16; 1024];
    let res = unsafe { GetModuleFileNameW(*CUR_HINSTANCE.lock().unwrap(), &mut path) } as usize;
    U16CString::from_vec(&path[0..res]).unwrap()
}

fn rust_apo_clsid_key() -> HSTRING {
    let guid = IRustAPO::IID;
    HSTRING::from(format!("SOFTWARE\\Classes\\CLSID\\{{{guid:?}}}"))
}

fn rust_inproc_key() -> HSTRING {
    let guid = IRustAPO::IID;
    HSTRING::from(format!(
        "SOFTWARE\\Classes\\CLSID\\{{{guid:?}}}\\InprocServer32"
    ))
}

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    let res = unsafe { RegisterAPO(*APO_PROPS) };
    if res.is_err() {
        return res;
    }

    let clsid_key = rust_apo_clsid_key();

    let mut clsid_key_handle = HKEY::default();
    unsafe {
        let err = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from(&clsid_key),
            0,
            None,
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_SET_VALUE | KEY_WOW64_64KEY,
            None,
            &mut clsid_key_handle,
            None,
        );

        if err.is_err() {
            return err.to_hresult();
        }
    }

    let empty_str = HSTRING::from(String::from(""));
    unsafe {
        let err = RegSetValueExW(
            clsid_key_handle,
            PCWSTR::from(&empty_str),
            0,
            REG_SZ,
            Some(as_u8_slice(&FRIENDLY_NAME)),
        );

        if err.is_err() {
            return err.to_hresult();
        }
        let err = RegCloseKey(clsid_key_handle);
        if err.is_err() {
            return err.to_hresult();
        }
    }

    // Set the inproc
    let mut inproc_key_handle = HKEY::default();
    let inproc_key = rust_inproc_key();
    unsafe {
        let err = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from(&inproc_key),
            0,
            None,
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_SET_VALUE | KEY_WOW64_64KEY,
            None,
            &mut inproc_key_handle,
            None,
        );

        if err.is_err() {
            return err.to_hresult();
        }

        let dll_location = get_dll_path().into_vec_with_nul();
        let err = RegSetValueExW(
            inproc_key_handle,
            PCWSTR::from(&empty_str),
            0,
            REG_SZ,
            Some(as_u8_slice(&dll_location)),
        );
        if err.is_err() {
            return err.to_hresult();
        }

        let thread_model = HSTRING::from(String::from("ThreadingModel"));
        let thread_model_val = U16CString::from_str("Both").unwrap().into_vec_with_nul();
        let err = RegSetValueExW(
            inproc_key_handle,
            PCWSTR::from(&thread_model),
            0,
            REG_SZ,
            Some(as_u8_slice(&thread_model_val)),
        );
        if err.is_err() {
            return err.to_hresult();
        }

        let err = RegCloseKey(inproc_key_handle);
        if err.is_err() {
            return err.to_hresult();
        }
    }

    S_OK
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    let guid = IRustAPO::IID;
    let inproc_key = HSTRING::from(format!(
        "SOFTWARE\\Classes\\CLSID\\{{{guid:?}}}\\InprocServer32"
    ));
    let clsid_key = HSTRING::from(format!("SOFTWARE\\Classes\\CLSID\\{{{guid:?}}}"));
    let res = unsafe {
        RegDeleteKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from(&inproc_key),
            KEY_WOW64_64KEY.0,
            0,
        )
    };
    if res.is_err() {
        return res.to_hresult();
    }
    let res = unsafe {
        RegDeleteKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from(&clsid_key),
            KEY_WOW64_64KEY.0,
            0,
        )
    };
    if res.is_err() {
        return res.to_hresult();
    }
    unsafe { UnregisterAPO(&IRustAPO::IID) }
}

#[derive(Debug)]
pub struct Apos {
    names: Vec<(String, GUID)>,
}

impl Apos {
    pub fn new() -> windows::core::Result<Self> {
        let mut apos = Self {
            names: Vec::default(),
        };
        let res = unsafe { EnumerateAPOs(Some(cb), (&mut apos) as *mut _ as *mut c_void) };
        if res.is_err() {
            Err(res.ok().unwrap_err())
        } else {
            Ok(apos)
        }
    }
}

unsafe extern "system" fn cb(
    pproperties: *mut APO_REG_PROPERTIES,
    pvrefdata: *mut c_void,
) -> HRESULT {
    let guid = (*pproperties).clsid;
    let friendly_name =
        widestring::U16CStr::from_ptr_str((*pproperties).szFriendlyName.as_ptr()).to_string_lossy();

    let p = &mut *(pvrefdata as *mut _ as *mut Apos);
    p.names.push((friendly_name, guid));
    S_OK
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    pub fn show_apos() {
        let apos = Apos::new();
        if let Ok(apos) = apos {
            println!("{apos:#?}. Count {} ", apos.names.len(),);
        } else {
            println!("{apos:?}");
        }
    }

    use super::*;
    #[test]
    fn show_apos_test() {
        show_apos();
    }

    #[test]
    fn check_interface() {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).unwrap();
        }
        {
            let apo: IUnknown =
                unsafe { CoCreateInstance(&IRustAPO::IID, None, CLSCTX_ALL).unwrap() };

            let res = apo.cast::<IRustAPO>();
            println!("{res:?}");
            if let Ok(r) = res {
                let mut o = 0;
                let _ = unsafe { r.MyFunction(&mut o) };
                println!("my val: {o}");
            }
        }
        unsafe {
            CoUninitialize();
        }
    }
}

fn as_u8_slice(slice: &[u16]) -> &[u8] {
    let len = 2 * slice.len();
    let ptr = slice.as_ptr().cast::<u8>();
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

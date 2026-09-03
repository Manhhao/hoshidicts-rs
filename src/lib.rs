mod ffi;
mod results;

pub use results::*;

use std::ffi::{CStr, CString, c_char, c_int};
use std::fmt;
use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::{NonNull, null, null_mut};

use results::slice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Failed,
    Import(String),
    Container(String),
    InteriorNul,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Failed => f.write_str("hoshidicts call failed"),
            Error::Import(e) => write!(f, "import failed: {e}"),
            Error::Container(e) => write!(f, "container operation failed: {e}"),
            Error::InteriorNul => f.write_str("string contains an interior nul byte"),
        }
    }
}

impl std::error::Error for Error {}

fn cstr(s: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|_| Error::InteriorNul)
}

fn cpath(path: &Path) -> Result<CString, Error> {
    CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_| Error::InteriorNul)
}

unsafe fn string(ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

pub struct Import {
    pub title: String,
    pub terms: u64,
    pub meta: u64,
    pub freq: u64,
    pub pitch: u64,
    pub kanji: u64,
    pub media: u64,
}

pub fn import(
    zip_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    low_ram: bool,
) -> Result<Import, Error> {
    let zip_path = cpath(zip_path.as_ref())?;
    let output_dir = cpath(output_dir.as_ref())?;

    let r = unsafe { ffi::hd_import(zip_path.as_ptr(), output_dir.as_ptr(), low_ram as c_int) };
    if r.is_null() {
        return Err(Error::Failed);
    }

    let result = unsafe {
        if ffi::hd_import_result_success(r) == 0 {
            Err(Error::Import(string(ffi::hd_import_result_error(r))))
        } else {
            Ok(Import {
                title: string(ffi::hd_import_result_title(r)),
                terms: ffi::hd_import_result_term_count(r),
                meta: ffi::hd_import_result_meta_count(r),
                freq: ffi::hd_import_result_freq_count(r),
                pitch: ffi::hd_import_result_pitch_count(r),
                kanji: ffi::hd_import_result_kanji_count(r),
                media: ffi::hd_import_result_media_count(r),
            })
        }
    };

    unsafe { ffi::hd_import_result_free(r) };
    result
}

unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let value = unsafe { string(ptr) };
    unsafe { ffi::hd_container_string_free(ptr) };
    Some(value)
}

unsafe fn take_error(ptr: *mut c_char) -> Error {
    unsafe { take_string(ptr) }.map_or(Error::Failed, Error::Container)
}

pub fn pack(dictionary_dir: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<u64, Error> {
    let dictionary_dir = cpath(dictionary_dir.as_ref())?;
    let output = output.as_ref();
    let output_path = cpath(output)?;

    let mut error = null_mut();
    if unsafe { ffi::hd_container_pack(dictionary_dir.as_ptr(), output_path.as_ptr(), &mut error) }
        != 0
    {
        return Err(unsafe { take_error(error) });
    }

    fs::metadata(output)
        .map(|m| m.len())
        .map_err(|_| Error::Failed)
}

pub fn verify(container: impl AsRef<Path>) -> Result<u32, Error> {
    let container = cpath(container.as_ref())?;

    let mut payload_version = 0;
    let mut error = null_mut();
    if unsafe { ffi::hd_container_verify(container.as_ptr(), &mut payload_version, &mut error) }
        != 0
    {
        return Err(unsafe { take_error(error) });
    }

    Ok(payload_version)
}

pub fn index(container: impl AsRef<Path>) -> Result<String, Error> {
    let container = cpath(container.as_ref())?;

    let mut json = null_mut();
    let mut error = null_mut();
    if unsafe { ffi::hd_container_index(container.as_ptr(), &mut json, &mut error) } != 0 {
        return Err(unsafe { take_error(error) });
    }

    unsafe { take_string(json) }.ok_or(Error::Failed)
}

unsafe impl Send for Deinflector {}
unsafe impl Send for Query {}
unsafe impl Send for Lookup<'_> {}
unsafe impl Send for OwnedLookup {}

pub struct Deinflector(NonNull<ffi::hd_deinflector>);

impl Deinflector {
    pub fn new() -> Self {
        Self(NonNull::new(unsafe { ffi::hd_deinflector_new() }).unwrap())
    }
}

impl Default for Deinflector {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Deinflector {
    fn drop(&mut self) {
        unsafe { ffi::hd_deinflector_free(self.0.as_ptr()) }
    }
}

pub struct Query(NonNull<ffi::hd_query>);

impl Query {
    pub fn new() -> Self {
        Self(NonNull::new(unsafe { ffi::hd_query_new() }).unwrap())
    }

    pub fn add_term_dict(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.add_dict(path.as_ref(), ffi::hd_query_add_term_dict)
    }

    pub fn add_freq_dict(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.add_dict(path.as_ref(), ffi::hd_query_add_freq_dict)
    }

    pub fn add_pitch_dict(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.add_dict(path.as_ref(), ffi::hd_query_add_pitch_dict)
    }

    pub fn add_kanji_dict(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.add_dict(path.as_ref(), ffi::hd_query_add_kanji_dict)
    }

    fn add_dict(
        &mut self,
        path: &Path,
        add: unsafe extern "C" fn(*mut ffi::hd_query, *const c_char) -> c_int,
    ) -> Result<(), Error> {
        let path = cpath(path)?;
        match unsafe { add(self.0.as_ptr(), path.as_ptr()) } {
            0 => Ok(()),
            _ => Err(Error::Failed),
        }
    }

    pub fn run(&self, expression: &str) -> Result<Results, Error> {
        let expression = cstr(expression)?;
        let mut terms = null();
        let mut count = 0;
        let ptr = unsafe {
            ffi::hd_query_run(self.0.as_ptr(), expression.as_ptr(), &mut terms, &mut count)
        };
        Ok(Results {
            ptr: NonNull::new(ptr).ok_or(Error::Failed)?,
            terms,
            count,
        })
    }

    pub fn run_kanji(&self, kanji: &str) -> Result<KanjiResults, Error> {
        let kanji = cstr(kanji)?;
        let mut entries = null();
        let mut count = 0;
        let ptr = unsafe {
            ffi::hd_query_run_kanji(self.0.as_ptr(), kanji.as_ptr(), &mut entries, &mut count)
        };
        Ok(KanjiResults {
            ptr: NonNull::new(ptr).ok_or(Error::Failed)?,
            entries,
            count,
        })
    }

    pub fn media_file(&self, dict_name: &str, media_path: &str) -> Option<&[u8]> {
        let dict_name = cstr(dict_name).ok()?;
        let media_path = cstr(media_path).ok()?;
        let file = unsafe {
            ffi::hd_query_get_media_file(self.0.as_ptr(), dict_name.as_ptr(), media_path.as_ptr())
        };
        (!file.data.is_null()).then(|| slice(file.data, file.size))
    }

    pub fn styles(&self) -> Result<Styles, Error> {
        let mut styles = null();
        let mut count = 0;
        let ptr = unsafe { ffi::hd_query_get_styles(self.0.as_ptr(), &mut styles, &mut count) };
        Ok(Styles {
            ptr: NonNull::new(ptr).ok_or(Error::Failed)?,
            styles,
            count,
        })
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe { ffi::hd_query_free(self.0.as_ptr()) }
    }
}

pub struct Lookup<'a>(
    NonNull<ffi::hd_lookup>,
    PhantomData<(&'a Query, &'a Deinflector)>,
);

impl<'a> Lookup<'a> {
    pub fn new(query: &'a Query, deinflector: &'a Deinflector) -> Self {
        let ptr = unsafe { ffi::hd_lookup_new(query.0.as_ptr(), deinflector.0.as_ptr()) };
        Self(NonNull::new(ptr).unwrap(), PhantomData)
    }

    pub fn run(
        &self,
        lookup_string: &str,
        max_results: c_int,
        scan_length: usize,
    ) -> Result<LookupResults, Error> {
        run_lookup(self.0.as_ptr(), lookup_string, max_results, scan_length)
    }

    pub fn run_with_options(
        &self,
        lookup_string: &str,
        max_results: c_int,
        scan_length: usize,
        options: &LookupOptions<'_>,
    ) -> Result<LookupResults, Error> {
        run_lookup_with_options(
            self.0.as_ptr(),
            lookup_string,
            max_results,
            scan_length,
            options,
        )
    }
}

impl Drop for Lookup<'_> {
    fn drop(&mut self) {
        unsafe { ffi::hd_lookup_free(self.0.as_ptr()) }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LookupFrequencyOrder {
    #[default]
    Auto,
    Ascending,
    Descending,
    Disabled,
}

impl LookupFrequencyOrder {
    fn to_ffi(self) -> ffi::hd_lookup_frequency_order {
        match self {
            LookupFrequencyOrder::Auto => ffi::hd_lookup_frequency_order::Auto,
            LookupFrequencyOrder::Ascending => ffi::hd_lookup_frequency_order::Ascending,
            LookupFrequencyOrder::Descending => ffi::hd_lookup_frequency_order::Descending,
            LookupFrequencyOrder::Disabled => ffi::hd_lookup_frequency_order::Disabled,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LookupOptions<'a> {
    pub frequency_dictionary: Option<&'a str>,
    pub frequency_order: LookupFrequencyOrder,
    pub primary_reading: Option<&'a str>,
}

fn hd_str(value: Option<&str>) -> ffi::hd_str {
    match value {
        Some(s) => ffi::hd_str {
            ptr: s.as_ptr().cast(),
            len: s.len(),
        },
        None => ffi::hd_str {
            ptr: null(),
            len: 0,
        },
    }
}

fn run_lookup(
    lookup: *const ffi::hd_lookup,
    lookup_string: &str,
    max_results: c_int,
    scan_length: usize,
) -> Result<LookupResults, Error> {
    let lookup_string = cstr(lookup_string)?;
    let mut results = null();
    let mut count = 0;
    let ptr = unsafe {
        ffi::hd_lookup_run(
            lookup,
            lookup_string.as_ptr(),
            max_results,
            scan_length,
            &mut results,
            &mut count,
        )
    };
    Ok(LookupResults {
        ptr: NonNull::new(ptr).ok_or(Error::Failed)?,
        results,
        count,
    })
}

fn run_lookup_with_options(
    lookup: *const ffi::hd_lookup,
    lookup_string: &str,
    max_results: c_int,
    scan_length: usize,
    options: &LookupOptions<'_>,
) -> Result<LookupResults, Error> {
    let lookup_string = cstr(lookup_string)?;
    let ffi_options = ffi::hd_lookup_options {
        frequency_dictionary: hd_str(options.frequency_dictionary),
        frequency_order: options.frequency_order.to_ffi() as c_int,
        primary_reading: hd_str(options.primary_reading),
    };
    let mut results = null();
    let mut count = 0;
    let ptr = unsafe {
        ffi::hd_lookup_run_with_options(
            lookup,
            lookup_string.as_ptr(),
            max_results,
            scan_length,
            &ffi_options,
            &mut results,
            &mut count,
        )
    };
    Ok(LookupResults {
        ptr: NonNull::new(ptr).ok_or(Error::Failed)?,
        results,
        count,
    })
}

pub struct OwnedLookup {
    lookup: NonNull<ffi::hd_lookup>,
    query: Query,
    _deinflector: Deinflector,
}

impl OwnedLookup {
    pub fn new(query: Query, deinflector: Deinflector) -> Self {
        let lookup = unsafe { ffi::hd_lookup_new(query.0.as_ptr(), deinflector.0.as_ptr()) };
        Self {
            lookup: NonNull::new(lookup).unwrap(),
            query,
            _deinflector: deinflector,
        }
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub fn run(
        &self,
        lookup_string: &str,
        max_results: c_int,
        scan_length: usize,
    ) -> Result<LookupResults, Error> {
        run_lookup(
            self.lookup.as_ptr(),
            lookup_string,
            max_results,
            scan_length,
        )
    }

    pub fn run_with_options(
        &self,
        lookup_string: &str,
        max_results: c_int,
        scan_length: usize,
        options: &LookupOptions<'_>,
    ) -> Result<LookupResults, Error> {
        run_lookup_with_options(
            self.lookup.as_ptr(),
            lookup_string,
            max_results,
            scan_length,
            options,
        )
    }
}

impl Drop for OwnedLookup {
    fn drop(&mut self) {
        unsafe { ffi::hd_lookup_free(self.lookup.as_ptr()) }
    }
}

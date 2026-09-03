#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int};

#[repr(C)]
pub struct hd_str {
    pub ptr: *const c_char,
    pub len: usize,
}

pub enum hd_import_result {}
pub enum hd_deinflector {}
pub enum hd_query {}
pub enum hd_results {}
pub enum hd_kanji_results {}
pub enum hd_styles {}
pub enum hd_lookup {}
pub enum hd_lookup_results {}

#[repr(C)]
pub struct hd_frequency {
    pub value: i32,
    pub display_value: hd_str,
}

#[repr(C)]
pub struct hd_dictionary_style {
    pub dict_name: hd_str,
    pub styles: hd_str,
}

#[repr(C)]
pub struct hd_media_file {
    pub data: *const u8,
    pub size: usize,
}

#[repr(C)]
pub struct hd_glossary_entry {
    pub dict_name: hd_str,
    pub glossary: hd_str,
    pub definition_tags: hd_str,
    pub term_tags: hd_str,
}

#[repr(C)]
pub struct hd_frequency_entry {
    pub dict_name: hd_str,
    pub frequencies: *const hd_frequency,
    pub frequencies_count: usize,
}

#[repr(C)]
pub struct hd_pitch {
    pub position: i32,
    pub pattern: hd_str,
    pub nasal: *const i32,
    pub nasal_count: usize,
    pub devoice: *const i32,
    pub devoice_count: usize,
}

#[repr(C)]
pub struct hd_pitch_entry {
    pub dict_name: hd_str,
    pub pitches: *const hd_pitch,
    pub pitches_count: usize,
    pub transcriptions: *const hd_str,
    pub transcriptions_count: usize,
}

#[repr(C)]
pub struct hd_term_result {
    pub expression: hd_str,
    pub reading: hd_str,
    pub rules: hd_str,
    pub score: i32,
    pub glossaries: *const hd_glossary_entry,
    pub glossaries_count: usize,
    pub frequencies: *const hd_frequency_entry,
    pub frequencies_count: usize,
    pub pitches: *const hd_pitch_entry,
    pub pitches_count: usize,
}

#[repr(C)]
pub struct hd_kanji_stat {
    pub key: hd_str,
    pub value: hd_str,
}

#[repr(C)]
pub struct hd_kanji_entry {
    pub dict_name: hd_str,
    pub onyomi: hd_str,
    pub kunyomi: hd_str,
    pub tags: hd_str,
    pub definitions: *const hd_str,
    pub definitions_count: usize,
    pub stats: *const hd_kanji_stat,
    pub stats_count: usize,
}

#[repr(C)]
pub struct hd_transform_group {
    pub name: hd_str,
    pub description: hd_str,
}

#[repr(C)]
pub struct hd_lookup_result {
    pub matched: hd_str,
    pub deinflected: hd_str,
    pub trace: *const hd_transform_group,
    pub trace_count: usize,
    pub term: hd_term_result,
    pub preprocessor_steps: i32,
}

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum hd_lookup_frequency_order {
    Auto = 0,
    Ascending = 1,
    Descending = 2,
    Disabled = 3,
}

#[repr(C)]
pub struct hd_lookup_options {
    pub frequency_dictionary: hd_str,
    pub frequency_order: i32,
    pub primary_reading: hd_str,
}

unsafe extern "C" {
    pub fn hd_import(
        zip_path: *const c_char,
        output_dir: *const c_char,
        low_ram: c_int,
    ) -> *mut hd_import_result;
    pub fn hd_import_result_free(r: *mut hd_import_result);
    pub fn hd_import_result_success(r: *const hd_import_result) -> c_int;
    pub fn hd_import_result_title(r: *const hd_import_result) -> *const c_char;
    pub fn hd_import_result_term_count(r: *const hd_import_result) -> u64;
    pub fn hd_import_result_meta_count(r: *const hd_import_result) -> u64;
    pub fn hd_import_result_freq_count(r: *const hd_import_result) -> u64;
    pub fn hd_import_result_pitch_count(r: *const hd_import_result) -> u64;
    pub fn hd_import_result_kanji_count(r: *const hd_import_result) -> u64;
    pub fn hd_import_result_media_count(r: *const hd_import_result) -> u64;
    pub fn hd_import_result_error(r: *const hd_import_result) -> *const c_char;

    pub fn hd_container_pack(
        dictionary_dir: *const c_char,
        output_path: *const c_char,
        error: *mut *mut c_char,
    ) -> c_int;
    pub fn hd_container_verify(
        container_path: *const c_char,
        payload_version: *mut u32,
        error: *mut *mut c_char,
    ) -> c_int;
    pub fn hd_container_index(
        container_path: *const c_char,
        index_json: *mut *mut c_char,
        error: *mut *mut c_char,
    ) -> c_int;
    pub fn hd_container_string_free(value: *mut c_char);

    pub fn hd_deinflector_new() -> *mut hd_deinflector;
    pub fn hd_deinflector_free(d: *mut hd_deinflector);

    pub fn hd_query_new() -> *mut hd_query;
    pub fn hd_query_free(q: *mut hd_query);
    pub fn hd_query_add_term_dict(q: *mut hd_query, path: *const c_char) -> c_int;
    pub fn hd_query_add_freq_dict(q: *mut hd_query, path: *const c_char) -> c_int;
    pub fn hd_query_add_pitch_dict(q: *mut hd_query, path: *const c_char) -> c_int;
    pub fn hd_query_add_kanji_dict(q: *mut hd_query, path: *const c_char) -> c_int;

    pub fn hd_query_run(
        q: *const hd_query,
        expression: *const c_char,
        out_terms: *mut *const hd_term_result,
        out_count: *mut usize,
    ) -> *mut hd_results;
    pub fn hd_results_free(r: *mut hd_results);

    pub fn hd_query_run_kanji(
        q: *const hd_query,
        kanji: *const c_char,
        out_entries: *mut *const hd_kanji_entry,
        out_count: *mut usize,
    ) -> *mut hd_kanji_results;
    pub fn hd_kanji_results_free(r: *mut hd_kanji_results);

    pub fn hd_query_get_media_file(
        q: *const hd_query,
        dict_name: *const c_char,
        media_path: *const c_char,
    ) -> hd_media_file;

    pub fn hd_query_get_styles(
        q: *const hd_query,
        out_styles: *mut *const hd_dictionary_style,
        out_count: *mut usize,
    ) -> *mut hd_styles;
    pub fn hd_styles_free(s: *mut hd_styles);

    pub fn hd_lookup_new(q: *mut hd_query, d: *mut hd_deinflector) -> *mut hd_lookup;
    pub fn hd_lookup_free(l: *mut hd_lookup);
    pub fn hd_lookup_run(
        l: *const hd_lookup,
        lookup_string: *const c_char,
        max_results: c_int,
        scan_length: usize,
        out_results: *mut *const hd_lookup_result,
        out_count: *mut usize,
    ) -> *mut hd_lookup_results;
    pub fn hd_lookup_run_with_options(
        l: *const hd_lookup,
        lookup_string: *const c_char,
        max_results: c_int,
        scan_length: usize,
        options: *const hd_lookup_options,
        out_results: *mut *const hd_lookup_result,
        out_count: *mut usize,
    ) -> *mut hd_lookup_results;
    pub fn hd_lookup_results_free(r: *mut hd_lookup_results);
}

#[cfg(test)]
mod abi {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn hd_str_layout() {
        assert_eq!(size_of::<hd_str>(), 2 * size_of::<usize>());
        assert_eq!(align_of::<hd_str>(), align_of::<usize>());
        assert_eq!(offset_of!(hd_str, ptr), 0);
        assert_eq!(offset_of!(hd_str, len), size_of::<usize>());
    }

    #[test]
    fn hd_lookup_options_layout() {
        let word = size_of::<usize>();
        assert_eq!(offset_of!(hd_lookup_options, frequency_dictionary), 0);
        assert_eq!(offset_of!(hd_lookup_options, frequency_order), 2 * word);
        assert_eq!(offset_of!(hd_lookup_options, primary_reading), 3 * word);
        assert_eq!(size_of::<hd_lookup_options>(), 5 * word);
        assert_eq!(align_of::<hd_lookup_options>(), align_of::<usize>());
    }

    #[test]
    fn frequency_order_discriminants() {
        assert_eq!(hd_lookup_frequency_order::Auto as i32, 0);
        assert_eq!(hd_lookup_frequency_order::Ascending as i32, 1);
        assert_eq!(hd_lookup_frequency_order::Descending as i32, 2);
        assert_eq!(hd_lookup_frequency_order::Disabled as i32, 3);
    }
}

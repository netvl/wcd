// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct Empty {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Empty {}

impl Empty {
    pub fn new() -> Empty {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Empty {
        static mut instance: ::protobuf::lazy::Lazy<Empty> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Empty,
        };
        unsafe {
            instance.get(Empty::new)
        }
    }
}

impl ::protobuf::Message for Empty {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Empty {
    fn new() -> Empty {
        Empty::new()
    }

    fn descriptor_static(_: ::std::option::Option<Empty>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<Empty>(
                    "Empty",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Empty {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Empty {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Empty {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PlaylistName {
    // message fields
    pub name: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PlaylistName {}

impl PlaylistName {
    pub fn new() -> PlaylistName {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PlaylistName {
        static mut instance: ::protobuf::lazy::Lazy<PlaylistName> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PlaylistName,
        };
        unsafe {
            instance.get(PlaylistName::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }
}

impl ::protobuf::Message for PlaylistName {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PlaylistName {
    fn new() -> PlaylistName {
        PlaylistName::new()
    }

    fn descriptor_static(_: ::std::option::Option<PlaylistName>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    PlaylistName::get_name_for_reflect,
                    PlaylistName::mut_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PlaylistName>(
                    "PlaylistName",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PlaylistName {
    fn clear(&mut self) {
        self.clear_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PlaylistName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PlaylistName {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct StatusInfo {
    // message fields
    pub playlists: ::std::collections::HashMap<::std::string::String, PlaylistInfo>,
    pub current_playlist: ::std::string::String,
    pub last_update: i64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for StatusInfo {}

impl StatusInfo {
    pub fn new() -> StatusInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static StatusInfo {
        static mut instance: ::protobuf::lazy::Lazy<StatusInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const StatusInfo,
        };
        unsafe {
            instance.get(StatusInfo::new)
        }
    }

    // repeated .proto.StatusInfo.PlaylistsEntry playlists = 1;

    pub fn clear_playlists(&mut self) {
        self.playlists.clear();
    }

    // Param is passed by value, moved
    pub fn set_playlists(&mut self, v: ::std::collections::HashMap<::std::string::String, PlaylistInfo>) {
        self.playlists = v;
    }

    // Mutable pointer to the field.
    pub fn mut_playlists(&mut self) -> &mut ::std::collections::HashMap<::std::string::String, PlaylistInfo> {
        &mut self.playlists
    }

    // Take field
    pub fn take_playlists(&mut self) -> ::std::collections::HashMap<::std::string::String, PlaylistInfo> {
        ::std::mem::replace(&mut self.playlists, ::std::collections::HashMap::new())
    }

    pub fn get_playlists(&self) -> &::std::collections::HashMap<::std::string::String, PlaylistInfo> {
        &self.playlists
    }

    fn get_playlists_for_reflect(&self) -> &::std::collections::HashMap<::std::string::String, PlaylistInfo> {
        &self.playlists
    }

    fn mut_playlists_for_reflect(&mut self) -> &mut ::std::collections::HashMap<::std::string::String, PlaylistInfo> {
        &mut self.playlists
    }

    // string current_playlist = 2;

    pub fn clear_current_playlist(&mut self) {
        self.current_playlist.clear();
    }

    // Param is passed by value, moved
    pub fn set_current_playlist(&mut self, v: ::std::string::String) {
        self.current_playlist = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_current_playlist(&mut self) -> &mut ::std::string::String {
        &mut self.current_playlist
    }

    // Take field
    pub fn take_current_playlist(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.current_playlist, ::std::string::String::new())
    }

    pub fn get_current_playlist(&self) -> &str {
        &self.current_playlist
    }

    fn get_current_playlist_for_reflect(&self) -> &::std::string::String {
        &self.current_playlist
    }

    fn mut_current_playlist_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.current_playlist
    }

    // int64 last_update = 3;

    pub fn clear_last_update(&mut self) {
        self.last_update = 0;
    }

    // Param is passed by value, moved
    pub fn set_last_update(&mut self, v: i64) {
        self.last_update = v;
    }

    pub fn get_last_update(&self) -> i64 {
        self.last_update
    }

    fn get_last_update_for_reflect(&self) -> &i64 {
        &self.last_update
    }

    fn mut_last_update_for_reflect(&mut self) -> &mut i64 {
        &mut self.last_update
    }
}

impl ::protobuf::Message for StatusInfo {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_map_into::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<PlaylistInfo>>(wire_type, is, &mut self.playlists)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.current_playlist)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int64()?;
                    self.last_update = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::compute_map_size::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<PlaylistInfo>>(1, &self.playlists);
        if !self.current_playlist.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.current_playlist);
        }
        if self.last_update != 0 {
            my_size += ::protobuf::rt::value_size(3, self.last_update, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        ::protobuf::rt::write_map_with_cached_sizes::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<PlaylistInfo>>(1, &self.playlists, os)?;
        if !self.current_playlist.is_empty() {
            os.write_string(2, &self.current_playlist)?;
        }
        if self.last_update != 0 {
            os.write_int64(3, self.last_update)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for StatusInfo {
    fn new() -> StatusInfo {
        StatusInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<StatusInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_map_accessor::<_, ::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<PlaylistInfo>>(
                    "playlists",
                    StatusInfo::get_playlists_for_reflect,
                    StatusInfo::mut_playlists_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "current_playlist",
                    StatusInfo::get_current_playlist_for_reflect,
                    StatusInfo::mut_current_playlist_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt64>(
                    "last_update",
                    StatusInfo::get_last_update_for_reflect,
                    StatusInfo::mut_last_update_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<StatusInfo>(
                    "StatusInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for StatusInfo {
    fn clear(&mut self) {
        self.clear_playlists();
        self.clear_current_playlist();
        self.clear_last_update();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for StatusInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for StatusInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PlaylistInfo {
    // message fields
    pub directories: ::protobuf::RepeatedField<::std::string::String>,
    pub files: ::protobuf::RepeatedField<::std::string::String>,
    pub total_files: u64,
    pub mode: ChangeMode,
    pub current_image: ::std::string::String,
    pub trigger_on_select: bool,
    pub use_last_on_select: bool,
    pub next_update: i64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PlaylistInfo {}

impl PlaylistInfo {
    pub fn new() -> PlaylistInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PlaylistInfo {
        static mut instance: ::protobuf::lazy::Lazy<PlaylistInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PlaylistInfo,
        };
        unsafe {
            instance.get(PlaylistInfo::new)
        }
    }

    // repeated string directories = 1;

    pub fn clear_directories(&mut self) {
        self.directories.clear();
    }

    // Param is passed by value, moved
    pub fn set_directories(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.directories = v;
    }

    // Mutable pointer to the field.
    pub fn mut_directories(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.directories
    }

    // Take field
    pub fn take_directories(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.directories, ::protobuf::RepeatedField::new())
    }

    pub fn get_directories(&self) -> &[::std::string::String] {
        &self.directories
    }

    fn get_directories_for_reflect(&self) -> &::protobuf::RepeatedField<::std::string::String> {
        &self.directories
    }

    fn mut_directories_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.directories
    }

    // repeated string files = 2;

    pub fn clear_files(&mut self) {
        self.files.clear();
    }

    // Param is passed by value, moved
    pub fn set_files(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.files = v;
    }

    // Mutable pointer to the field.
    pub fn mut_files(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.files
    }

    // Take field
    pub fn take_files(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.files, ::protobuf::RepeatedField::new())
    }

    pub fn get_files(&self) -> &[::std::string::String] {
        &self.files
    }

    fn get_files_for_reflect(&self) -> &::protobuf::RepeatedField<::std::string::String> {
        &self.files
    }

    fn mut_files_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.files
    }

    // uint64 total_files = 3;

    pub fn clear_total_files(&mut self) {
        self.total_files = 0;
    }

    // Param is passed by value, moved
    pub fn set_total_files(&mut self, v: u64) {
        self.total_files = v;
    }

    pub fn get_total_files(&self) -> u64 {
        self.total_files
    }

    fn get_total_files_for_reflect(&self) -> &u64 {
        &self.total_files
    }

    fn mut_total_files_for_reflect(&mut self) -> &mut u64 {
        &mut self.total_files
    }

    // .proto.ChangeMode mode = 4;

    pub fn clear_mode(&mut self) {
        self.mode = ChangeMode::SEQUENTIAL;
    }

    // Param is passed by value, moved
    pub fn set_mode(&mut self, v: ChangeMode) {
        self.mode = v;
    }

    pub fn get_mode(&self) -> ChangeMode {
        self.mode
    }

    fn get_mode_for_reflect(&self) -> &ChangeMode {
        &self.mode
    }

    fn mut_mode_for_reflect(&mut self) -> &mut ChangeMode {
        &mut self.mode
    }

    // string current_image = 5;

    pub fn clear_current_image(&mut self) {
        self.current_image.clear();
    }

    // Param is passed by value, moved
    pub fn set_current_image(&mut self, v: ::std::string::String) {
        self.current_image = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_current_image(&mut self) -> &mut ::std::string::String {
        &mut self.current_image
    }

    // Take field
    pub fn take_current_image(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.current_image, ::std::string::String::new())
    }

    pub fn get_current_image(&self) -> &str {
        &self.current_image
    }

    fn get_current_image_for_reflect(&self) -> &::std::string::String {
        &self.current_image
    }

    fn mut_current_image_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.current_image
    }

    // bool trigger_on_select = 6;

    pub fn clear_trigger_on_select(&mut self) {
        self.trigger_on_select = false;
    }

    // Param is passed by value, moved
    pub fn set_trigger_on_select(&mut self, v: bool) {
        self.trigger_on_select = v;
    }

    pub fn get_trigger_on_select(&self) -> bool {
        self.trigger_on_select
    }

    fn get_trigger_on_select_for_reflect(&self) -> &bool {
        &self.trigger_on_select
    }

    fn mut_trigger_on_select_for_reflect(&mut self) -> &mut bool {
        &mut self.trigger_on_select
    }

    // bool use_last_on_select = 7;

    pub fn clear_use_last_on_select(&mut self) {
        self.use_last_on_select = false;
    }

    // Param is passed by value, moved
    pub fn set_use_last_on_select(&mut self, v: bool) {
        self.use_last_on_select = v;
    }

    pub fn get_use_last_on_select(&self) -> bool {
        self.use_last_on_select
    }

    fn get_use_last_on_select_for_reflect(&self) -> &bool {
        &self.use_last_on_select
    }

    fn mut_use_last_on_select_for_reflect(&mut self) -> &mut bool {
        &mut self.use_last_on_select
    }

    // int64 next_update = 8;

    pub fn clear_next_update(&mut self) {
        self.next_update = 0;
    }

    // Param is passed by value, moved
    pub fn set_next_update(&mut self, v: i64) {
        self.next_update = v;
    }

    pub fn get_next_update(&self) -> i64 {
        self.next_update
    }

    fn get_next_update_for_reflect(&self) -> &i64 {
        &self.next_update
    }

    fn mut_next_update_for_reflect(&mut self) -> &mut i64 {
        &mut self.next_update
    }
}

impl ::protobuf::Message for PlaylistInfo {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.directories)?;
                },
                2 => {
                    ::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.files)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.total_files = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.mode = tmp;
                },
                5 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.current_image)?;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.trigger_on_select = tmp;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.use_last_on_select = tmp;
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int64()?;
                    self.next_update = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.directories {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        for value in &self.files {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        if self.total_files != 0 {
            my_size += ::protobuf::rt::value_size(3, self.total_files, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.mode != ChangeMode::SEQUENTIAL {
            my_size += ::protobuf::rt::enum_size(4, self.mode);
        }
        if !self.current_image.is_empty() {
            my_size += ::protobuf::rt::string_size(5, &self.current_image);
        }
        if self.trigger_on_select != false {
            my_size += 2;
        }
        if self.use_last_on_select != false {
            my_size += 2;
        }
        if self.next_update != 0 {
            my_size += ::protobuf::rt::value_size(8, self.next_update, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.directories {
            os.write_string(1, &v)?;
        };
        for v in &self.files {
            os.write_string(2, &v)?;
        };
        if self.total_files != 0 {
            os.write_uint64(3, self.total_files)?;
        }
        if self.mode != ChangeMode::SEQUENTIAL {
            os.write_enum(4, self.mode.value())?;
        }
        if !self.current_image.is_empty() {
            os.write_string(5, &self.current_image)?;
        }
        if self.trigger_on_select != false {
            os.write_bool(6, self.trigger_on_select)?;
        }
        if self.use_last_on_select != false {
            os.write_bool(7, self.use_last_on_select)?;
        }
        if self.next_update != 0 {
            os.write_int64(8, self.next_update)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PlaylistInfo {
    fn new() -> PlaylistInfo {
        PlaylistInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<PlaylistInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "directories",
                    PlaylistInfo::get_directories_for_reflect,
                    PlaylistInfo::mut_directories_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "files",
                    PlaylistInfo::get_files_for_reflect,
                    PlaylistInfo::mut_files_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "total_files",
                    PlaylistInfo::get_total_files_for_reflect,
                    PlaylistInfo::mut_total_files_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<ChangeMode>>(
                    "mode",
                    PlaylistInfo::get_mode_for_reflect,
                    PlaylistInfo::mut_mode_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "current_image",
                    PlaylistInfo::get_current_image_for_reflect,
                    PlaylistInfo::mut_current_image_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "trigger_on_select",
                    PlaylistInfo::get_trigger_on_select_for_reflect,
                    PlaylistInfo::mut_trigger_on_select_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "use_last_on_select",
                    PlaylistInfo::get_use_last_on_select_for_reflect,
                    PlaylistInfo::mut_use_last_on_select_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt64>(
                    "next_update",
                    PlaylistInfo::get_next_update_for_reflect,
                    PlaylistInfo::mut_next_update_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PlaylistInfo>(
                    "PlaylistInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PlaylistInfo {
    fn clear(&mut self) {
        self.clear_directories();
        self.clear_files();
        self.clear_total_files();
        self.clear_mode();
        self.clear_current_image();
        self.clear_trigger_on_select();
        self.clear_use_last_on_select();
        self.clear_next_update();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PlaylistInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PlaylistInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum ChangeMode {
    SEQUENTIAL = 0,
    RANDOM = 1,
}

impl ::protobuf::ProtobufEnum for ChangeMode {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<ChangeMode> {
        match value {
            0 => ::std::option::Option::Some(ChangeMode::SEQUENTIAL),
            1 => ::std::option::Option::Some(ChangeMode::RANDOM),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [ChangeMode] = &[
            ChangeMode::SEQUENTIAL,
            ChangeMode::RANDOM,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<ChangeMode>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("ChangeMode", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for ChangeMode {
}

impl ::std::default::Default for ChangeMode {
    fn default() -> Self {
        ChangeMode::SEQUENTIAL
    }
}

impl ::protobuf::reflect::ProtobufValue for ChangeMode {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0fproto/wcd.proto\x12\x05proto\"\x07\n\x05Empty\"\"\n\x0cPlaylistNam\
    e\x12\x12\n\x04name\x18\x01\x20\x01(\tR\x04name\"\xeb\x01\n\nStatusInfo\
    \x12>\n\tplaylists\x18\x01\x20\x03(\x0b2\x20.proto.StatusInfo.PlaylistsE\
    ntryR\tplaylists\x12)\n\x10current_playlist\x18\x02\x20\x01(\tR\x0fcurre\
    ntPlaylist\x12\x1f\n\x0blast_update\x18\x03\x20\x01(\x03R\nlastUpdate\
    \x1aQ\n\x0ePlaylistsEntry\x12\x10\n\x03key\x18\x01\x20\x01(\tR\x03key\
    \x12)\n\x05value\x18\x02\x20\x01(\x0b2\x13.proto.PlaylistInfoR\x05value:\
    \x028\x01\"\xad\x02\n\x0cPlaylistInfo\x12\x20\n\x0bdirectories\x18\x01\
    \x20\x03(\tR\x0bdirectories\x12\x14\n\x05files\x18\x02\x20\x03(\tR\x05fi\
    les\x12\x1f\n\x0btotal_files\x18\x03\x20\x01(\x04R\ntotalFiles\x12%\n\
    \x04mode\x18\x04\x20\x01(\x0e2\x11.proto.ChangeModeR\x04mode\x12#\n\rcur\
    rent_image\x18\x05\x20\x01(\tR\x0ccurrentImage\x12*\n\x11trigger_on_sele\
    ct\x18\x06\x20\x01(\x08R\x0ftriggerOnSelect\x12+\n\x12use_last_on_select\
    \x18\x07\x20\x01(\x08R\x0fuseLastOnSelect\x12\x1f\n\x0bnext_update\x18\
    \x08\x20\x01(\x03R\nnextUpdate*(\n\nChangeMode\x12\x0e\n\nSEQUENTIAL\x10\
    \0\x12\n\n\x06RANDOM\x10\x012\xf8\x01\n\x03Wcd\x12-\n\rTriggerChange\x12\
    \x0c.proto.Empty\x1a\x0c.proto.Empty\"\0\x120\n\x10RefreshPlaylists\x12\
    \x0c.proto.Empty\x1a\x0c.proto.Empty\"\0\x12)\n\tTerminate\x12\x0c.proto\
    .Empty\x1a\x0c.proto.Empty\"\0\x12.\n\tGetStatus\x12\x0c.proto.Empty\x1a\
    \x11.proto.StatusInfo\"\0\x125\n\x0eChangePlaylist\x12\x13.proto.Playlis\
    tName\x1a\x0c.proto.Empty\"\0b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}

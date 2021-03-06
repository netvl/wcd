// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

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


// interface

pub trait Wcd {
    fn trigger_change(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty>;

    fn trigger_update(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty>;

    fn refresh_playlists(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty>;

    fn terminate(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty>;

    fn get_status(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::StatusInfo>;

    fn change_playlist(&self, o: ::grpc::RequestOptions, p: super::wcd::PlaylistName) -> ::grpc::SingleResponse<super::wcd::Empty>;

    fn get_statistics(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::StatsInfo>;
}

// client

pub struct WcdClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_TriggerChange: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::Empty, super::wcd::Empty>>,
    method_TriggerUpdate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::Empty, super::wcd::Empty>>,
    method_RefreshPlaylists: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::Empty, super::wcd::Empty>>,
    method_Terminate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::Empty, super::wcd::Empty>>,
    method_GetStatus: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::Empty, super::wcd::StatusInfo>>,
    method_ChangePlaylist: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::PlaylistName, super::wcd::Empty>>,
    method_GetStatistics: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wcd::Empty, super::wcd::StatsInfo>>,
}

impl ::grpc::ClientStub for WcdClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        WcdClient {
            grpc_client: grpc_client,
            method_TriggerChange: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/TriggerChange".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TriggerUpdate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/TriggerUpdate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_RefreshPlaylists: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/RefreshPlaylists".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Terminate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/Terminate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetStatus: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/GetStatus".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ChangePlaylist: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/ChangePlaylist".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetStatistics: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/wcd.Wcd/GetStatistics".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Wcd for WcdClient {
    fn trigger_change(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty> {
        self.grpc_client.call_unary(o, p, self.method_TriggerChange.clone())
    }

    fn trigger_update(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty> {
        self.grpc_client.call_unary(o, p, self.method_TriggerUpdate.clone())
    }

    fn refresh_playlists(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty> {
        self.grpc_client.call_unary(o, p, self.method_RefreshPlaylists.clone())
    }

    fn terminate(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::Empty> {
        self.grpc_client.call_unary(o, p, self.method_Terminate.clone())
    }

    fn get_status(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::StatusInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetStatus.clone())
    }

    fn change_playlist(&self, o: ::grpc::RequestOptions, p: super::wcd::PlaylistName) -> ::grpc::SingleResponse<super::wcd::Empty> {
        self.grpc_client.call_unary(o, p, self.method_ChangePlaylist.clone())
    }

    fn get_statistics(&self, o: ::grpc::RequestOptions, p: super::wcd::Empty) -> ::grpc::SingleResponse<super::wcd::StatsInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetStatistics.clone())
    }
}

// server

pub struct WcdServer;


impl WcdServer {
    pub fn new_service_def<H : Wcd + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/wcd.Wcd",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/TriggerChange".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.trigger_change(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/TriggerUpdate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.trigger_update(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/RefreshPlaylists".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.refresh_playlists(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/Terminate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.terminate(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/GetStatus".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_status(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/ChangePlaylist".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.change_playlist(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/wcd.Wcd/GetStatistics".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_statistics(o, p))
                    },
                ),
            ],
        )
    }
}

pub mod smm {
    pub mod users {
        tonic::include_proto!("proto.users.v1");
    }
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("smm_descriptor");
}

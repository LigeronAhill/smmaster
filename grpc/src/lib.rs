pub mod smm {
    tonic::include_proto!("proto.smm.v1");
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("smm_descriptor");
}

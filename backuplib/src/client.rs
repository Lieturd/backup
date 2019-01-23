use grpc::RequestOptions;

use crate::proto::baacup_grpc;
use crate::proto::baacup_grpc::Baacup as GrpcBaacup;
use crate::proto::baacup;
use crate::rpc::*;

pub struct BaacupClient(baacup_grpc::BaacupClient);

impl grpc::ClientStub for BaacupClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        BaacupClient(<baacup_grpc::BaacupClient as grpc::ClientStub>::with_client(grpc_client))
    }
}

impl Baacup for BaacupClient {
    fn init_upload(&self, metadata: FileMetadata) -> Result<u32, String> {
        let mut file_metadata = baacup::FileMetadata::new();
        file_metadata.set_file_name(metadata.file_name.into());
        file_metadata.set_last_modified(metadata.last_modified);
        file_metadata.set_file_size(metadata.file_size);

        let token_resp = self.0.init_upload(RequestOptions::new(), file_metadata);
        let mut token = token_resp.wait_drop_metadata().unwrap();
        match token.get_status() {
            baacup::Status::SUCCESS => Ok(token.get_token().get_token()),
            baacup::Status::ERROR => Err(token.take_error_message()),
        }
    }

    fn get_head(&self, token: u32) -> Result<u64, String> {
        let mut upload_token = baacup::UploadToken::new();
        upload_token.set_token(token);

        let head_resp = self.0.get_head(RequestOptions::new(), upload_token);
        let mut head = head_resp.wait_drop_metadata().unwrap();
        match head.get_status() {
            baacup::Status::SUCCESS => Ok(head.get_offset()),
            baacup::Status::ERROR => Err(head.take_error_message()),
        }
    }

    fn upload_chunk(&self, chunk: FileChunk) -> Result<u32, String> {
        let mut file_chunk = baacup::FileChunk::new();
        file_chunk.set_token(chunk.token);
        file_chunk.set_offset(chunk.offset);
        file_chunk.set_data(chunk.data);

        let checksum_resp = self.0.upload_chunk(RequestOptions::new(), file_chunk);
        let mut checksum = checksum_resp.wait_drop_metadata().unwrap();
        match checksum.get_status() {
            baacup::Status::SUCCESS => Ok(checksum.get_checksum()),
            baacup::Status::ERROR => Err(checksum.take_error_message()),
        }
    }

    fn file_is_uploaded(&self, _p: FileMetadata) -> Result<bool, String> {
        unimplemented!()
    }
}

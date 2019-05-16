use grpc::RequestOptions;
use futures::Future;

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
    fn init_upload(&self, metadata: FileMetadata) -> BaacupFuture<u32> {
        let mut file_metadata = baacup::FileMetadata::new();
        file_metadata.set_file_name(metadata.file_name.into());
        file_metadata.set_last_modified(metadata.last_modified);
        file_metadata.set_file_size(metadata.file_size);

        let token_resp = self.0.init_upload(RequestOptions::new(), file_metadata);
        BaacupFuture::new(token_resp.drop_metadata()
            .then(|token_result|
                token_result.map_err(|e| e.to_string()).and_then(|mut token|
                    match token.get_status() {
                        baacup::Status::SUCCESS => Ok(token.get_token().get_token()),
                        baacup::Status::ERROR => Err(token.take_error_message()),
                    }
                )
            )
        )
    }

    fn get_head(&self, token: u32) -> BaacupFuture<u64> {
        let mut upload_token = baacup::UploadToken::new();
        upload_token.set_token(token);

        let head_resp = self.0.get_head(RequestOptions::new(), upload_token);
        BaacupFuture::new(head_resp.drop_metadata()
            .then(|head_result|
                head_result.map_err(|e| e.to_string()).and_then(|mut head|
                    match head.get_status() {
                        baacup::Status::SUCCESS => Ok(head.get_offset()),
                        baacup::Status::ERROR => Err(head.take_error_message()),
                    }
                )
            )
        )
    }

    fn upload_chunk(&self, chunk: FileChunk) -> BaacupFuture<u32> {
        let mut file_chunk = baacup::FileChunk::new();
        file_chunk.set_token(chunk.token);
        file_chunk.set_offset(chunk.offset);
        file_chunk.set_data(chunk.data);

        let checksum_resp = self.0.upload_chunk(RequestOptions::new(), file_chunk);
        BaacupFuture::new(checksum_resp.drop_metadata()
            .then(|checksum_result|
                checksum_result.map_err(|e| e.to_string()).and_then(|mut checksum|
                    match checksum.get_status() {
                        baacup::Status::SUCCESS => Ok(checksum.get_checksum()),
                        baacup::Status::ERROR => Err(checksum.take_error_message()),
                    }
                )
            )
        )
    }

    fn file_is_uploaded(&self, metadata: FileMetadata) -> BaacupFuture<bool> {
        let mut file_metadata = baacup::FileMetadata::new();
        file_metadata.set_file_name(metadata.file_name.into());
        file_metadata.set_last_modified(metadata.last_modified);
        file_metadata.set_file_size(metadata.file_size);

        let is_uploaded_resp = self.0.file_is_uploaded(RequestOptions::new(), file_metadata);
        BaacupFuture::new(is_uploaded_resp.drop_metadata()
            .then(|is_uploaded_result|
                is_uploaded_result.map_err(|e| e.to_string()).and_then(|mut is_uploaded|
                    match is_uploaded.get_status() {
                        baacup::Status::SUCCESS => Ok(is_uploaded.get_file_is_uploaded()),
                        baacup::Status::ERROR => Err(is_uploaded.take_error_message()),
                    }
                )
            )
        )
    }

    fn download_chunk(&self, metadata: FileMetadata, offset: u64) -> BaacupFuture<Vec<u8>> {
        let mut download_chunk_info = baacup::DownloadChunkInfo::new();

        let mut file_metadata = baacup::FileMetadata::new();
        file_metadata.set_file_name(metadata.file_name.into());
        file_metadata.set_last_modified(metadata.last_modified);
        file_metadata.set_file_size(metadata.file_size);

        download_chunk_info.set_metadata(file_metadata);
        download_chunk_info.set_offset(offset);

        let download_chunk_resp = self.0.download_chunk(RequestOptions::new(), download_chunk_info);
        BaacupFuture::new(download_chunk_resp.drop_metadata()
            .then(|download_chunk_result|
                download_chunk_result.map_err(|e| e.to_string()).and_then(|mut data|
                    match data.get_status() {
                        baacup::Status::SUCCESS => Ok(data.take_data()),
                        baacup::Status::ERROR => Err(data.take_error_message()),
                    }
                )
            )
        )
    }
}

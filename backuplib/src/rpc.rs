use futures::{IntoFuture, Future, Poll};

use crate::proto::baacup;
use crate::proto::baacup_grpc;
pub use crate::proto::baacup_grpc::BaacupServer;

pub struct FileMetadata {
    pub file_name: String,
    pub last_modified: u32,
    pub file_size: u64,
}

pub struct FileChunk {
    pub token: u32,
    pub offset: u64,
    pub data: Vec<u8>,
}

pub struct BaacupFuture<T: Send + 'static>(Box<dyn Future<Item = T, Error = String> + Send>);

impl<T> BaacupFuture<T>
    where T: Send + 'static,
{
    pub fn new<F>(future: F) -> BaacupFuture<T>
        where F: IntoFuture<Item = T, Error = String>,
              F::Future: Send + 'static,
    {
        BaacupFuture(Box::new(future.into_future()))
    }
}

impl<T> Future for BaacupFuture<T>
    where T: Send,
{
    type Item = T;
    type Error = String;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

pub trait Baacup {
    fn init_upload(&self, metadata: FileMetadata) -> BaacupFuture<u32>;
    fn get_head(&self, token: u32) -> BaacupFuture<u64>;
    fn upload_chunk(&self, chunk: FileChunk) -> BaacupFuture<u32>;
    fn file_is_uploaded(&self, metadata: FileMetadata) -> BaacupFuture<bool>;
}

impl<T> baacup_grpc::Baacup for T
    where T: Baacup
{
    fn init_upload(&self, _o: grpc::RequestOptions, mut p: baacup::FileMetadata) -> grpc::SingleResponse<baacup::InitUploadResponse> {
        let metadata = FileMetadata {
            file_name: p.take_file_name(),
            last_modified: p.get_last_modified(),
            file_size: p.get_file_size(),
        };

        grpc::SingleResponse::no_metadata(Baacup::init_upload(self, metadata)
            .then(|future_result| {
                match future_result {
                    Ok(token) => {
                        let mut init_upload_response = baacup::InitUploadResponse::new();
                        init_upload_response.set_status(baacup::Status::SUCCESS);
                        init_upload_response.mut_token().set_token(token);
                        Ok(init_upload_response)
                    }
                    Err(error) => {
                        let mut init_upload_response = baacup::InitUploadResponse::new();
                        init_upload_response.set_status(baacup::Status::ERROR);
                        init_upload_response.set_error_message(error);
                        Ok(init_upload_response)
                    }
                }
            })
        )
    }

    fn get_head(&self, _o: grpc::RequestOptions, p: baacup::UploadToken) -> grpc::SingleResponse<baacup::FileHead> {
        let token = p.get_token();

        grpc::SingleResponse::no_metadata(Baacup::get_head(self, token)
            .then(|future_result| {
                match future_result {
                    Ok(offset) => {
                        let mut file_head = baacup::FileHead::new();
                        file_head.set_status(baacup::Status::SUCCESS);
                        file_head.set_offset(offset);
                        Ok(file_head)
                    }
                    Err(error) => {
                        let mut file_head = baacup::FileHead::new();
                        file_head.set_status(baacup::Status::ERROR);
                        file_head.set_error_message(error);
                        Ok(file_head)
                    }
                }
            })
        )
    }

    fn upload_chunk(&self, _o: grpc::RequestOptions, mut p: baacup::FileChunk) -> grpc::SingleResponse<baacup::UploadFileResponse> {
        let file_chunk = FileChunk {
            token: p.get_token(),
            offset: p.get_offset(),
            data: p.take_data(),
        };

        grpc::SingleResponse::no_metadata(Baacup::upload_chunk(self, file_chunk)
            .then(|future_result| {
                match future_result {
                    Ok(checksum) => {
                        let mut upload_file_response = baacup::UploadFileResponse::new();
                        upload_file_response.set_status(baacup::Status::SUCCESS);
                        upload_file_response.set_checksum(checksum);
                        Ok(upload_file_response)
                    }
                    Err(error) => {
                        let mut upload_file_response = baacup::UploadFileResponse::new();
                        upload_file_response.set_status(baacup::Status::ERROR);
                        upload_file_response.set_error_message(error);
                        Ok(upload_file_response)
                    }
                }
            })
        )
    }

    fn file_is_uploaded(&self, _o: grpc::RequestOptions, _p: baacup::FileMetadata) -> grpc::SingleResponse<baacup::FileIsUploadedResponse> {
        unimplemented!()
    }
}

use crate::proto::baacup;
use crate::proto::baacup_grpc;

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

pub trait Baacup {
    fn init_upload(&self, metadata: FileMetadata) -> Result<u32, String>;
    fn get_head(&self, token: u32) -> Result<u64, String>;
    fn upload_chunk(&self, chunk: FileChunk) -> Result<u32, String>;
    fn file_is_uploaded(&self, metadata: FileMetadata) -> Result<bool, String>;
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
        match Baacup::init_upload(self, metadata) {
            Ok(token) => {
                let mut init_upload_response = baacup::InitUploadResponse::new();
                init_upload_response.set_status(baacup::Status::SUCCESS);
                init_upload_response.mut_token().set_token(token);
                grpc::SingleResponse::completed(init_upload_response)
            }
            Err(error) => {
                let mut init_upload_response = baacup::InitUploadResponse::new();
                init_upload_response.set_status(baacup::Status::ERROR);
                init_upload_response.set_error_message(error);
                grpc::SingleResponse::completed(init_upload_response)
            }
        }
    }

    fn get_head(&self, _o: grpc::RequestOptions, p: baacup::UploadToken) -> grpc::SingleResponse<baacup::FileHead> {
        let token = p.get_token();
        match Baacup::get_head(self, token) {
            Ok(offset) => {
                let mut file_head = baacup::FileHead::new();
                file_head.set_status(baacup::Status::SUCCESS);
                file_head.set_offset(offset);
                grpc::SingleResponse::completed(file_head)
            }
            Err(error) => {
                let mut file_head = baacup::FileHead::new();
                file_head.set_status(baacup::Status::ERROR);
                file_head.set_error_message(error);
                grpc::SingleResponse::completed(file_head)
            }
        }
    }

    fn upload_chunk(&self, _o: grpc::RequestOptions, mut p: baacup::FileChunk) -> grpc::SingleResponse<baacup::UploadFileResponse> {
        let file_chunk = FileChunk {
            token: p.get_token(),
            offset: p.get_offset(),
            data: p.take_data(),
        };
        match Baacup::upload_chunk(self, file_chunk) {
            Ok(checksum) => {
                let mut upload_file_response = baacup::UploadFileResponse::new();
                upload_file_response.set_status(baacup::Status::SUCCESS);
                upload_file_response.set_checksum(checksum);
                grpc::SingleResponse::completed(upload_file_response)
            }
            Err(error) => {
                let mut upload_file_response = baacup::UploadFileResponse::new();
                upload_file_response.set_status(baacup::Status::ERROR);
                upload_file_response.set_error_message(error);
                grpc::SingleResponse::completed(upload_file_response)
            }
        }
    }

    fn file_is_uploaded(&self, _o: grpc::RequestOptions, _p: baacup::FileMetadata) -> grpc::SingleResponse<baacup::FileIsUploadedResponse> {
        unimplemented!()
    }
}

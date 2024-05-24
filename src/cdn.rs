use actix_web::HttpResponse;
use serde_repr::*;
use std::{
    collections::HashMap, fs::{self, OpenOptions}, io::{Seek, SeekFrom, Write}, sync::RwLock
};
use leptos::ServerFnError;

pub const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MiB

#[repr(u8)]
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
pub enum CdnError {
    AlreadyExists,
    RecipeDoesntExist,
    UnsupportedImageFormat,
    InvalidName,
    InternalError,
}

#[allow(clippy::from_over_into)] // Don't need the reverse
impl Into<ServerFnError> for CdnError {
    fn into(self) -> ServerFnError {
        match self {
            CdnError::AlreadyExists => ServerFnError::Args(String::from("Already Exists")),
            CdnError::RecipeDoesntExist => ServerFnError::Args(String::from("Recipe doesn't exist")),
            CdnError::UnsupportedImageFormat => ServerFnError::Args(String::from("Unsupported image format")),
            CdnError::InternalError => ServerFnError::new("Internal server error"),
            CdnError::InvalidName => ServerFnError::Args(String::from("Inavlid name")),
        }
    }
}

#[allow(clippy::from_over_into)] // Don't need the reverse
impl Into<HttpResponse> for CdnError {
    fn into(self) -> HttpResponse {
        match self {
            CdnError::AlreadyExists => HttpResponse::Conflict().finish(),
            CdnError::RecipeDoesntExist => HttpResponse::NotFound().finish(),
            CdnError::UnsupportedImageFormat => HttpResponse::NotImplemented().finish(),
            CdnError::InternalError => HttpResponse::InternalServerError().finish(),
            CdnError::InvalidName => HttpResponse::BadRequest().finish(),
        }
    }
}

#[derive(Debug)]
pub struct Cdn {
    path: String,
    // Key is recipe name, value is next image id
    meta: RwLock<HashMap<String, u32>>,
    meta_file_handle: RwLock<fs::File>,
}

impl Cdn {
    /// Panics on fs error
    pub fn new(path: &str) -> Self {
        let meta_path = format!("{path}meta.cbor");

        let meta = RwLock::new({
            if fs::metadata(&meta_path).is_ok() {
                let file = fs::File::open(&meta_path).unwrap();
                ciborium::from_reader(file).unwrap()
            } else {
                fs::create_dir_all(path).unwrap();
                let file = fs::File::create(&meta_path).unwrap();
                let new_meta = HashMap::new();
                ciborium::into_writer(&new_meta, file).unwrap();
                new_meta
            }
        });

        Self{
            path: path.to_owned(),
            meta,
            meta_file_handle: RwLock::new(OpenOptions::new()
                .read(false)
                .write(true)
                .create(false)
                .open(&meta_path)
                .unwrap()),
        }
    }

    pub fn transaction<F>(&self, callback: F) -> Result<(), CdnError>
    where F: FnOnce(&CdnTransactionManager) -> Result<(), CdnError>, {
        let manager = CdnTransactionManager {
            path: &self.path,
            meta: &self.meta,
            meta_file_handle: &self.meta_file_handle,
        };

        callback(&manager)?;
        manager.dump_meta()
    }
}

#[derive(Debug)]
pub struct CdnTransactionManager<'a> {
    path: &'a str,
    // Key is recipe name, value is next image id
    meta: &'a RwLock<HashMap<String, u32>>,
    meta_file_handle: &'a RwLock<fs::File>,
}

impl<'a> CdnTransactionManager<'a> {
    pub fn dump_meta(&self) -> Result<(), CdnError> {
        let meta_handle = self.meta.read()
            .map_err(|_| CdnError::InternalError)?;

        let mut file_handle = self.meta_file_handle.write()
            .map_err(|_| CdnError::InternalError)?;
        file_handle.set_len(0)
            .map_err(|_| CdnError::InternalError)?;
        file_handle.seek(SeekFrom::Start(0))
            .map_err(|_| CdnError::InternalError)?;

        ciborium::into_writer(&*meta_handle, &*file_handle)
            .map_err(|_| CdnError::InternalError)
    }

    pub fn create_recipe(&self, recipe_name: &str) -> Result<(), CdnError> {
        let already_exists = {
            self.meta.read()
                .map_err(|_| CdnError::InternalError)?
                .contains_key(recipe_name)
        };
        if already_exists {
            return Err(CdnError::AlreadyExists);
        }

        fs::create_dir(format!("{}{}", self.path, recipe_name))
            .map_err(|_| CdnError::InternalError)?;

        self.meta.write()
            .map_err(|_| CdnError::InternalError)?
            .insert(String::from(recipe_name), 0);

        Ok(())
    }

    pub fn delete_recipe(&self, recipe_name: &str) -> Result<(), CdnError> {
        let exists = {
            self.meta.read()
                .inspect(|x| println!("{x:#?}"))
                .map_err(|_| CdnError::InternalError)?
                .contains_key(recipe_name)
        };
        if !exists {
            return Err(CdnError::RecipeDoesntExist);
        }

        fs::remove_dir_all(format!("{}{}", self.path, recipe_name))
            .map_err(|_| CdnError::InternalError)?;

        self.meta.write()
            .map_err(|_| CdnError::InternalError)?
            .remove(recipe_name);

        Ok(())
    }

    pub fn upload_icon(&self, recipe_name: &str, icon_data: &[u8]) -> Result<(), CdnError> {
        let exists = {
            self.meta.read()
                .inspect(|x| println!("{x:#?}"))
                .map_err(|_| CdnError::InternalError)?
                .contains_key(recipe_name)
        };
        if !exists {
            return Err(CdnError::RecipeDoesntExist);
        }

        let webp_buffer = convert_to_webp(icon_data)?;
        let mut file = fs::File::create(format!("{}{recipe_name}/icon", self.path))
            .map_err(|_| CdnError::InternalError)?;
        file.write_all(&webp_buffer)
            .map_err(|_| CdnError::InternalError)?;

        Ok(())
    }
}

fn convert_to_webp(buffer: &[u8]) -> Result<Vec<u8>, CdnError> {
    use image::ImageFormat;
    use webp::Encoder;

    let format = image::guess_format(buffer)
        .map_err(|_| CdnError::UnsupportedImageFormat)?;
    
    if format == ImageFormat::WebP {
        return Ok(buffer.to_vec());
    }

    let img = image::load_from_memory(buffer)
        .map_err(|_| CdnError::UnsupportedImageFormat)?;

    let encoder = Encoder::from_image(&img)
        .map_err(|_| CdnError::UnsupportedImageFormat)?;
    let webp_data = encoder.encode(70.0); // 100.0 for maximum quality

    Ok(webp_data.to_vec())
}

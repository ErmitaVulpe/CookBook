use actix_web::HttpResponse;
use serde_repr::*;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    fmt,
    io::{Seek, SeekFrom, Write},
    sync::RwLock,
};

pub const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MiB

#[repr(u8)]
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
pub enum CdnError {
    AlreadyExists,
    RecipeDoesntExist,
    ImageDoesntExist,
    UnsupportedImageFormat,
    InvalidName,
    InternalError,
}

impl fmt::Display for CdnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            CdnError::AlreadyExists => "Already exists",
            CdnError::RecipeDoesntExist => "Recipe doesn't exist",
            CdnError::ImageDoesntExist => "image doesn't exist",
            CdnError::UnsupportedImageFormat => "Unsupported image format",
            CdnError::InvalidName => "Invalid name",
            CdnError::InternalError => "Internal error",
        })
    }
}

impl std::error::Error for CdnError {}

#[allow(clippy::from_over_into)] // Don't need the reverse
impl Into<HttpResponse> for CdnError {
    fn into(self) -> HttpResponse {
        match self {
            CdnError::AlreadyExists => HttpResponse::Conflict().finish(),
            CdnError::RecipeDoesntExist => HttpResponse::NotFound().finish(),
            CdnError::ImageDoesntExist => HttpResponse::NotFound().finish(),
            CdnError::UnsupportedImageFormat => HttpResponse::NotImplemented().finish(),
            CdnError::InternalError => HttpResponse::InternalServerError().finish(),
            CdnError::InvalidName => HttpResponse::BadRequest().finish(),
        }
    }
}

#[derive(Debug)]
pub struct Cdn {
    pub path: String,
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

    pub fn get_image_list(&self, recipe_name: &str) -> Result<Vec<String>, CdnError> {
        let entries = fs::read_dir(format!("{}{}", &self.path, recipe_name.to_lowercase()))
            .map_err(|_| CdnError::RecipeDoesntExist)?;
        
        let mut image_list = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| CdnError::InternalError)?;
            let path = entry.path();

            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    image_list.push(file_name_str.to_string());
                }
            }
        }

        Ok(image_list)
    }

    pub fn delete_images(&self, recipe_name: &str, image_names: &[String]) -> Result<(), CdnError> {
        for name in image_names {
            let file_path = format!("{}{}/{}", &self.path, recipe_name, name);
            if fs::metadata(&file_path).is_err() {
                return Err(CdnError::ImageDoesntExist);
            }
            fs::remove_file(&file_path)
                .map_err(|_| CdnError::InternalError)?;
        }

        Ok(())
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
        let recipe_name = recipe_name.to_lowercase();

        let already_exists = {
            self.meta.read()
                .map_err(|_| CdnError::InternalError)?
                .contains_key(&recipe_name)
        };
        if already_exists {
            return Err(CdnError::AlreadyExists);
        }

        fs::create_dir(format!("{}{}", self.path, &recipe_name))
            .map_err(|_| CdnError::InternalError)?;

        self.meta.write()
            .map_err(|_| CdnError::InternalError)?
            .insert(String::from(&recipe_name), 0);

        Ok(())
    }

    pub fn delete_recipe(&self, recipe_name: &str) -> Result<(), CdnError> {
        let exists = {
            self.meta.read()
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

    pub fn upload_image(&self, recipe_name: &str, image_data: &[u8]) -> Result<(), CdnError> {
        let exists = {
            self.meta.read()
                .map_err(|_| CdnError::InternalError)?
                .contains_key(recipe_name)
        };
        if !exists {
            return Err(CdnError::RecipeDoesntExist);
        }

        let next_index = {
            let mut write_lock = self.meta.write()
                .map_err(|_| CdnError::InternalError)?;
            
            let mut next_index = 0;
            write_lock.entry(recipe_name.to_string())
                .and_modify(|i| { 
                    next_index = *i;
                    *i += 1;
                });
            
            next_index
        };

        let webp_buffer = convert_to_webp(image_data)?;
        let mut file = fs::File::create(format!("{}{recipe_name}/{next_index}", self.path))
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
    let webp_data = encoder.encode(70.0);

    Ok(webp_data.to_vec())
}

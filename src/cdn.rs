use serde_repr::*;
use std::{collections::HashMap, fs, sync::RwLock};

#[derive(Debug)] // TEMP
pub struct Cdn {
    path: String,
    meta: RwLock<(u32, HashMap<u32, u32>)>,
}

impl Cdn {
    pub fn new(path: &str) -> Result<Self, ()> {
        let meta = RwLock::new({
            let meta_path = format!("{path}meta.cbor");
            if fs::metadata(&meta_path).is_ok() {
                let file = fs::File::open(&meta_path).map_err(|_|())?;
                ciborium::from_reader(file).unwrap()
            } else {
                fs::create_dir(&path).map_err(|_|())?;
                let file = fs::File::create(&meta_path).map_err(|_|())?;
                let new_meta = (0, HashMap::new());
                ciborium::into_writer(&new_meta, file).unwrap();
                new_meta
            }

        });

        Ok(Self{
            path: path.to_owned(),
            meta,
        })
    }

    pub fn create_recipe(&self, recipe_id: u32, ) {
        
    }

    pub fn add_photo(&self, recipe_id: u32, extension: FileExtensions) -> Result<(), ()> {
        Ok(())
    }

    // REMBER: 0 must always exist (icon)
    // TODO create new recipe meta
    // TODO remove photo meta
    // TODO remove recipe meta
    // TODO upload photo
    // TODO remove photo
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum FileExtensions {
    Jpg,
    Png,
    Webp,
}

impl TryFrom<&str> for FileExtensions {
    type Error = ();
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "jpg" | "jpeg" => Self::Jpg,
            "png" => Self::Png,
            "webp" => Self::Webp,
            _ => return Err(()),
        })
    }
}

impl Into<&str> for FileExtensions {
    fn into(self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }
}

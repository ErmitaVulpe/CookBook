use serde_repr::*;
use std::ops::Deref;
use std::{collections::HashMap, fs, sync::RwLock};
use std::io::{BufReader, BufRead};
use ciborium::from_reader;

type NextID = u32;

#[derive(Debug)] // TEMP
pub struct Cdn {
    path: String,
    meta: RwLock<(
        NextID,
        HashMap<
            u32, (
                NextID,
                HashMap<
                    u32,
                    FileExtensions
                >
            )
        >
    )>,
}

impl Cdn {
    pub fn new(path: &str) -> Self {
        let meta = RwLock::new({
            let mut map = HashMap::new();
            
            let mut max_recipe_id = -1i64;
            for dir in fs::read_dir(&path).unwrap() {
                let dir = dir.unwrap();
                let recipe_id = dir
                    .file_name()
                    .into_string()
                    .unwrap()
                    .parse::<u32>()
                    .unwrap();

                if recipe_id as i64 > max_recipe_id {
                    max_recipe_id = recipe_id as i64
                }

                let file = fs::File::open(format!("{path}{recipe_id}/meta.cbor")).unwrap();
                let map_entry: (NextID, HashMap<u32, FileExtensions>) = from_reader(file).unwrap();
                map.insert(recipe_id, map_entry);
            }

            (max_recipe_id as u32 + 1, map)


            // for dir in fs::read_dir(&path).unwrap() {
            //     let dir = dir.unwrap();
            //     let recipe_id = dir
            //         .file_name()
            //         .into_string()
            //         .unwrap()
            //         .parse::<u32>()
            //         .unwrap();
                
            //     let mut inner_map = HashMap::new();
                
            //     let images = fs::read_dir(dir.path()).unwrap()
            //         .map(|x| x.unwrap().file_name().into_string().unwrap())
            //         .skip_while(|x| x == "meta.cbor");

            //     for img in images {
            //         let mut split = img.split(".");
            //         let img_id = split.next().unwrap().parse::<u32>().unwrap();
            //         let extension = FileExtensions::try_from(split.next().unwrap()).unwrap();
            //         inner_map.insert(img_id, extension);
            //     }

            //     map.insert(recipe_id, inner_map);
            // }
        });

        println!("{meta:#?}");

        Self {
            path: path.to_owned(),
            meta,
        }
    }

    pub fn add_photo_entry(&self, recipe_id: u32, extension: FileExtensions) -> Result<(), ()> {
        let mut lock = self.meta.write().map_err(|_| ())?;
        let recipe = match lock.1.get_mut(&recipe_id) {
            Some(val) => val,
            None => {
                let new_entry = (0, HashMap::new());
                let next_id = lock.0;
                lock.1.insert(next_id, new_entry);
                lock.0 += 1;
                let reff = lock.1.get_mut(&next_id).unwrap();
                reff
            },
        };
        recipe.1.insert(recipe.0, extension);
        recipe.0 += 1;

        let mut file = fs::File::create(format!("{}{recipe_id}/meta.cbor", self.path)).map_err(|_| ())?;
        ciborium::ser::into_writer(recipe, &mut file).map_err(|_| ())?;
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

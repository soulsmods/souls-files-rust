use std::{collections::HashMap, io::Cursor};

use bevy::{
    app::{App, Plugin},
    asset::{io::Reader, AssetLoader, BoxedFuture, Handle, LoadContext},
    prelude::{Asset, AssetApp, Reflect},
};
use fstools_formats::bnd4::BND4;
use futures_lite::AsyncReadExt;

pub struct Bnd4Plugin;

impl Plugin for Bnd4Plugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Archive>()
            .register_type::<Archive>()
            .init_asset::<ArchiveEntry>()
            .register_type::<ArchiveEntry>()
            .register_asset_loader(Bnd4Loader);
    }
}

pub struct Bnd4Loader;

#[derive(Asset, Debug, Reflect)]
pub struct ArchiveEntry {
    pub data: Vec<u8>,
}

#[derive(Asset, Debug, Default, Reflect)]
pub struct Archive {
    pub files: HashMap<String, Handle<ArchiveEntry>>,
}

impl AssetLoader for Bnd4Loader {
    type Asset = Archive;
    type Settings = ();
    type Error = std::io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut archive = Archive::default();

            let mut data = Vec::new();
            reader.read_to_end(&mut data).await?;

            let bnd = BND4::from_reader(Cursor::new(&data))?;
            for file in bnd.files {
                let handle = load_context.labeled_asset_scope(file.path.clone(), |_| {
                    let file_offset = file.data_offset as usize;
                    let file_end = file_offset + file.compressed_size as usize;
                    let file_data = data[file_offset..file_end].to_vec();

                    ArchiveEntry { data: file_data }
                });

                archive.files.insert(file.path, handle);
            }

            Ok(archive)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bnd.dcx", "bnd"]
    }
}

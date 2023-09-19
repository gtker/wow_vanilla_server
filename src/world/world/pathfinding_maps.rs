use namigator::raw::{build_bvh, build_map, bvh_files_exist, map_files_exist};
use namigator::vanilla::{Map, VanillaMap};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PathfindingMaps {
    maps: HashMap<Map, VanillaMap>,
}

impl PathfindingMaps {
    pub fn new() -> Self {
        let maps = if let Some(data_path) = std::option_env!("WOW_VANILLA_USE_MAPS") {
            let output = std::env::temp_dir().join("wow_vanilla_server");
            println!("Building and using maps for pathfind from data directory '{data_path}' and outputting to '{}'. This may take a while.", output.to_str().unwrap());
            let mut m = HashMap::new();

            let threads = {
                let t = std::thread::available_parallelism().unwrap().get() as u32;
                let t = t.saturating_sub(2);
                if t == 0 {
                    1
                } else {
                    t
                }
            };

            if !bvh_files_exist(&output).unwrap() {
                println!("Building gameobjects.");
                build_bvh(data_path, &output, threads).unwrap();
                println!("Gameobjects built.");
            } else {
                println!("Gameobjects already built.");
            }

            const MAP: Map = Map::DevelopmentLand;

            if !map_files_exist(&output, MAP.directory_name()).unwrap() {
                println!("Building map {MAP} ({})", MAP.directory_name());
                build_map(data_path, &output, MAP.directory_name(), "", threads).unwrap();
                println!("Finished building {MAP} ({})", MAP.directory_name());
            } else {
                println!("{MAP} ({}) already built.", MAP.directory_name());
            }

            let mut v =
                VanillaMap::build_gameobjects_and_map(data_path, &output, MAP, threads).unwrap();
            v.load_all_adts().unwrap();
            m.insert(MAP, v);
            println!("Finished setting up maps");

            m
        } else {
            println!("Not using maps for pathfind.");
            HashMap::new()
        };

        Self { maps }
    }

    pub fn get(&self, map: &Map) -> Option<&VanillaMap> {
        self.maps.get(&map)
    }
}

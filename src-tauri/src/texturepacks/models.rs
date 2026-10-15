use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Seek};
use zip::ZipArchive;

const MODELS_ROOT: &str = "assets/minecraft/models/";
const BLOCK_MODELS: &str = "assets/minecraft/models/block/";
const MAX_PARENTS: usize = 12;
const MAX_REFERENCES: usize = 12;
const DIRECTIONS: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelFace {
    pub direction: String,
    /// The texture of the face, like `block/stone`.
    pub texture: String,
    /// Left, top, right and bottom of the part of the texture, in sixteenths.
    pub uv: Option<[f64; 4]>,
    pub rotation: i64,
    /// Grass and leaves are drawn in grey and coloured by the biome.
    pub tinted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelRotation {
    pub origin: [f64; 3],
    pub axis: String,
    pub angle: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelElement {
    pub from: [f64; 3],
    pub to: [f64; 3],
    pub rotation: Option<ModelRotation>,
    pub faces: Vec<ModelFace>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockModel {
    pub elements: Vec<ModelElement>,
}

fn plain(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

fn load<R: Read + Seek>(jar: &mut ZipArchive<R>, name: &str) -> Option<Value> {
    let mut entry = jar.by_name(&format!("{MODELS_ROOT}{}.json", plain(name))).ok()?;
    let mut text = String::new();
    entry.read_to_string(&mut text).ok()?;
    serde_json::from_str(&text).ok()
}

/// What a model says itself and what it takes over from its parents: the texture names and the shapes.
struct Resolved {
    textures: HashMap<String, String>,
    elements: Vec<Value>,
}

fn resolve<R: Read + Seek>(jar: &mut ZipArchive<R>, name: &str) -> Option<Resolved> {
    let mut textures = HashMap::new();
    let mut elements = None;
    let mut next = Some(name.to_owned());
    for step in 0..MAX_PARENTS {
        let Some(current) = next.take() else { break };
        let Some(model) = load(jar, &current) else {
            if step == 0 {
                return None;
            }
            break;
        };
        if let Some(own) = model["textures"].as_object() {
            for (key, value) in own {
                if let Some(value) = value.as_str() {
                    textures.entry(key.clone()).or_insert_with(|| value.to_owned());
                }
            }
        }
        if elements.is_none() {
            elements = model["elements"].as_array().cloned();
        }
        next = model["parent"].as_str().map(str::to_owned);
    }
    Some(Resolved { textures, elements: elements.unwrap_or_default() })
}

/// Follows `#name` references until a real texture is reached.
fn texture_of(textures: &HashMap<String, String>, reference: &str) -> Option<String> {
    let mut value = reference;
    for _ in 0..MAX_REFERENCES {
        match value.strip_prefix('#') {
            Some(key) => value = textures.get(key)?,
            None => return Some(plain(value).to_owned()),
        }
    }
    None
}

fn numbers<const N: usize>(value: &Value) -> Option<[f64; N]> {
    let list = value.as_array().filter(|list| list.len() == N)?;
    let mut result = [0.0; N];
    for (slot, item) in result.iter_mut().zip(list) {
        *slot = item.as_f64()?;
    }
    Some(result)
}

fn element_from(value: &Value, textures: &HashMap<String, String>) -> Option<ModelElement> {
    let faces: Vec<ModelFace> = DIRECTIONS
        .iter()
        .filter_map(|direction| {
            let face = value["faces"].get(*direction)?;
            Some(ModelFace {
                direction: (*direction).to_owned(),
                texture: texture_of(textures, face["texture"].as_str()?)?,
                uv: numbers(&face["uv"]),
                rotation: face["rotation"].as_i64().unwrap_or(0),
                tinted: face.get("tintindex").is_some(),
            })
        })
        .collect();
    if faces.is_empty() {
        return None;
    }
    let rotation = value.get("rotation").and_then(|turn| {
        Some(ModelRotation { origin: numbers(&turn["origin"])?, axis: turn["axis"].as_str()?.to_owned(), angle: turn["angle"].as_f64()? })
    });
    Some(ModelElement { from: numbers(&value["from"])?, to: numbers(&value["to"])?, rotation, faces })
}

fn model_named<R: Read + Seek>(jar: &mut ZipArchive<R>, name: &str) -> Option<BlockModel> {
    let resolved = resolve(jar, name)?;
    let elements: Vec<ModelElement> = resolved.elements.iter().filter_map(|element| element_from(element, &resolved.textures)).collect();
    (!elements.is_empty()).then_some(BlockModel { elements })
}

fn uses(model: &BlockModel, texture: &str) -> bool {
    model.elements.iter().any(|element| element.faces.iter().any(|face| face.texture == texture))
}

/// The shortening names of a texture: `furnace_front_on` is tried as itself, then `furnace_front`, then `furnace`.
fn candidates(name: &str) -> Vec<String> {
    let mut found = vec![name.to_owned()];
    let mut rest = name;
    while let Some((shorter, _)) = rest.rsplit_once('_') {
        found.push(shorter.to_owned());
        rest = shorter;
    }
    found
}

/// The shape a block texture is drawn on in the game, such as the six different sides of a crafting table.
/// `texture` is the resource name without `.png`, for example `block/crafting_table_top`.
pub fn block_model<R: Read + Seek>(jar: &mut ZipArchive<R>, texture: &str) -> Option<BlockModel> {
    let name = texture.strip_prefix("block/")?;
    for candidate in candidates(name) {
        if let Some(model) = model_named(jar, &format!("block/{candidate}")).filter(|model| uses(model, texture)) {
            return Some(model);
        }
    }
    let names: Vec<String> = jar
        .file_names()
        .flatten()
        .filter_map(|file| file.strip_prefix(BLOCK_MODELS)?.strip_suffix(".json").map(str::to_owned))
        .collect();
    names.into_iter().find_map(|other| model_named(jar, &format!("block/{other}")).filter(|model| uses(model, texture)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    fn jar(models: &[(&str, &str)]) -> ZipArchive<Cursor<Vec<u8>>> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, json) in models {
            writer.start_file(format!("{MODELS_ROOT}{name}.json"), SimpleFileOptions::default()).unwrap();
            writer.write_all(json.as_bytes()).unwrap();
        }
        ZipArchive::new(writer.finish().unwrap()).unwrap()
    }

    const CUBE: &str = r##"{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{
        "down":{"texture":"#down"},"up":{"texture":"#up"},"north":{"texture":"#north"},
        "south":{"texture":"#south"},"west":{"texture":"#west"},"east":{"texture":"#east"}}}]}"##;

    fn crafting_table() -> ZipArchive<Cursor<Vec<u8>>> {
        jar(&[
            ("block/cube", CUBE),
            (
                "block/crafting_table",
                r#"{"parent":"minecraft:block/cube","textures":{"down":"minecraft:block/oak_planks","up":"block/crafting_table_top",
                    "north":"block/crafting_table_front","south":"block/crafting_table_front","west":"block/crafting_table_side","east":"block/crafting_table_side"}}"#,
            ),
        ])
    }

    fn texture_on(model: &BlockModel, direction: &str) -> String {
        model.elements[0].faces.iter().find(|face| face.direction == direction).unwrap().texture.clone()
    }

    #[test]
    fn a_block_texture_is_drawn_on_the_whole_block_with_all_its_sides() {
        let model = block_model(&mut crafting_table(), "block/crafting_table_top").unwrap();

        assert_eq!(model.elements.len(), 1);
        assert_eq!(texture_on(&model, "up"), "block/crafting_table_top");
        assert_eq!(texture_on(&model, "north"), "block/crafting_table_front");
        assert_eq!(texture_on(&model, "east"), "block/crafting_table_side");
        assert_eq!(texture_on(&model, "down"), "block/oak_planks");
    }

    #[test]
    fn finds_a_model_that_has_another_name_than_the_texture() {
        let mut archive = jar(&[
            ("block/cube", CUBE),
            ("block/lamp", r#"{"parent":"block/cube","textures":{"down":"block/a","up":"block/odd_name","north":"block/a","south":"block/a","west":"block/a","east":"block/a"}}"#),
        ]);

        let model = block_model(&mut archive, "block/odd_name").unwrap();

        assert_eq!(texture_on(&model, "up"), "block/odd_name");
        assert!(block_model(&mut archive, "block/nowhere").is_none());
        assert!(block_model(&mut archive, "item/apple").is_none());
    }

    #[test]
    fn keeps_rotations_uv_and_tint_of_the_shapes() {
        let mut archive = jar(&[(
            "block/plant",
            r##"{"textures":{"p":"block/fern"},"elements":[{"from":[0.8,0,8],"to":[15.2,16,8],
                "rotation":{"origin":[8,8,8],"axis":"y","angle":45,"rescale":true},
                "faces":{"north":{"uv":[0,0,16,16],"texture":"#p","tintindex":0},"south":{"texture":"#p","rotation":90}}}]}"##,
        )]);

        let model = block_model(&mut archive, "block/fern").unwrap();

        let element = &model.elements[0];
        assert_eq!(element.rotation, Some(ModelRotation { origin: [8.0, 8.0, 8.0], axis: "y".into(), angle: 45.0 }));
        let north = &element.faces[0];
        assert_eq!((north.uv, north.tinted), (Some([0.0, 0.0, 16.0, 16.0]), true));
        assert_eq!((element.faces[1].rotation, element.faces[1].uv), (90, None));
    }

    #[test]
    fn survives_models_that_refer_to_themselves() {
        let mut archive = jar(&[
            ("block/loop", r##"{"parent":"block/loop","textures":{"a":"#b","b":"#a"},"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"up":{"texture":"#a"}}}]}"##),
        ]);

        assert!(block_model(&mut archive, "block/loop").is_none());
    }
}

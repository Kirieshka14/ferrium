use std::fs;
use std::path::Path;

use ferrium_math::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    SceneRoot,
    Camera,
    DirectionalLight,
    Group,
    Mesh,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshKind {
    // 3D Models
    Cube,
    Plane,
    Sphere,
    Cylinder,
    Pyramid,
    Torus,
    ObjModel,
    // 2D Models
    Quad2D,
    Circle2D,
    Triangle2D,
    Star2D,
    Hexagon2D,
    Sprite2D,
}

impl MeshKind {
    pub fn is_2d(self) -> bool {
        matches!(
            self,
            MeshKind::Quad2D
                | MeshKind::Circle2D
                | MeshKind::Triangle2D
                | MeshKind::Star2D
                | MeshKind::Hexagon2D
                | MeshKind::Sprite2D
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            MeshKind::Cube => "Cube (3D)",
            MeshKind::Plane => "Plane (3D)",
            MeshKind::Sphere => "Sphere (3D)",
            MeshKind::Cylinder => "Cylinder (3D)",
            MeshKind::Pyramid => "Pyramid (3D)",
            MeshKind::Torus => "Torus (3D)",
            MeshKind::ObjModel => "OBJ Model (3D)",
            MeshKind::Quad2D => "Quad (2D)",
            MeshKind::Circle2D => "Circle (2D)",
            MeshKind::Triangle2D => "Triangle (2D)",
            MeshKind::Star2D => "Star (2D)",
            MeshKind::Hexagon2D => "Hexagon (2D)",
            MeshKind::Sprite2D => "Sprite (2D)",
        }
    }

    pub const ALL: &'static [MeshKind] = &[
        MeshKind::Cube,
        MeshKind::Sphere,
        MeshKind::Cylinder,
        MeshKind::Pyramid,
        MeshKind::Torus,
        MeshKind::Plane,
        MeshKind::ObjModel,
        MeshKind::Quad2D,
        MeshKind::Circle2D,
        MeshKind::Triangle2D,
        MeshKind::Star2D,
        MeshKind::Hexagon2D,
        MeshKind::Sprite2D,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamDef {
    Neutral,
    Player,
    Enemy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_ferrium(&self) -> Color {
        Color::rgba(self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshFilterDef {
    pub mesh: MeshKind,
    #[serde(default)]
    pub asset_path: Option<String>,
    #[serde(default)]
    pub wireframe: bool,
}

impl MeshFilterDef {
    pub fn simple(mesh: MeshKind) -> Self {
        Self {
            mesh,
            asset_path: None,
            wireframe: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRendererDef {
    pub material_name: String,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxColliderDef {
    pub is_trigger: bool,
    pub size: [f32; 3],
    pub center: [f32; 3],
    pub team: TeamDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDef {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptDef {
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntityDef {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub name: String,
    pub kind: EntityKind,
    pub active: bool,
    pub tag: String,
    pub layer: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub mesh_filter: Option<MeshFilterDef>,
    pub mesh_renderer: Option<MeshRendererDef>,
    pub box_collider: Option<BoxColliderDef>,
    pub health: Option<HealthDef>,
    pub script: Option<ScriptDef>,
}

impl SceneEntityDef {
    pub fn new_cube(id: u64, parent_id: Option<u64>, name: impl Into<String>, pos: [f32; 3]) -> Self {
        Self::new_model(
            id,
            parent_id,
            name,
            MeshKind::Cube,
            None,
            pos,
            RgbaColor::new(72, 82, 100, 255),
        )
    }

    pub fn new_model(
        id: u64,
        parent_id: Option<u64>,
        name: impl Into<String>,
        mesh: MeshKind,
        asset_path: Option<String>,
        pos: [f32; 3],
        color: RgbaColor,
    ) -> Self {
        let is_2d = mesh.is_2d();
        let mat = if is_2d {
            "Sprite2D_Mat"
        } else {
            "Default_Material"
        };
        let col_size = if is_2d {
            [1.0, 1.0, 0.12]
        } else {
            [1.0, 1.0, 1.0]
        };
        Self {
            id,
            parent_id,
            name: name.into(),
            kind: EntityKind::Mesh,
            active: true,
            tag: "Untagged".into(),
            layer: if is_2d { "UI" } else { "Default" }.into(),
            position: pos,
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            mesh_filter: Some(MeshFilterDef {
                mesh,
                asset_path,
                wireframe: false,
            }),
            mesh_renderer: Some(MeshRendererDef {
                material_name: mat.into(),
                color,
            }),
            box_collider: Some(BoxColliderDef {
                is_trigger: false,
                size: col_size,
                center: [0.0, 0.0, 0.0],
                team: TeamDef::Neutral,
            }),
            health: None,
            script: None,
        }
    }

    pub fn new_empty(id: u64, parent_id: Option<u64>, name: impl Into<String>, kind: EntityKind) -> Self {
        Self {
            id,
            parent_id,
            name: name.into(),
            kind,
            active: true,
            tag: "Untagged".into(),
            layer: "Default".into(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            mesh_filter: None,
            mesh_renderer: None,
            box_collider: None,
            health: None,
            script: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFile {
    pub name: String,
    pub next_entity_id: u64,
    pub entities: Vec<SceneEntityDef>,
}

impl SceneFile {
    pub fn allocate_id(&mut self) -> u64 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    pub fn empty() -> Self {
        let mut s = Self {
            name: "UntitledScene".into(),
            next_entity_id: 1,
            entities: Vec::new(),
        };
        let root = s.allocate_id();
        s.entities
            .push(SceneEntityDef::new_empty(root, None, "UntitledScene", EntityKind::SceneRoot));
        s
    }

    /// Default scene strictly matching the reference mockup Hierarchy and 3D viewport.
    pub fn sample_scene() -> Self {
        let mut s = Self {
            name: "SampleScene".into(),
            next_entity_id: 1,
            entities: Vec::new(),
        };
        let root_id = s.allocate_id();
        s.entities.push(SceneEntityDef::new_empty(
            root_id,
            None,
            "SampleScene",
            EntityKind::SceneRoot,
        ));

        let cam_id = s.allocate_id();
        let mut cam = SceneEntityDef::new_empty(cam_id, Some(root_id), "MainCamera", EntityKind::Camera);
        cam.tag = "MainCamera".into();
        cam.position = [0.0, 1.8, 3.8];
        s.entities.push(cam);

        let light_id = s.allocate_id();
        let mut light = SceneEntityDef::new_empty(
            light_id,
            Some(root_id),
            "DirectionalLight",
            EntityKind::DirectionalLight,
        );
        light.position = [0.0, 3.0, 0.0];
        light.rotation = [50.0, -30.0, 0.0];
        s.entities.push(light);

        let env_id = s.allocate_id();
        s.entities.push(SceneEntityDef::new_empty(
            env_id,
            Some(root_id),
            "Environment",
            EntityKind::Group,
        ));

        let ground_id = s.allocate_id();
        let mut ground =
            SceneEntityDef::new_empty(ground_id, Some(env_id), "Ground", EntityKind::Mesh);
        ground.scale = [12.0, 1.0, 12.0];
        ground.mesh_filter = Some(MeshFilterDef::simple(MeshKind::Plane));
        ground.mesh_renderer = Some(MeshRendererDef {
            material_name: "GridFloor_Mat".into(),
            color: RgbaColor::new(26, 31, 42, 95),
        });
        s.entities.push(ground);

        let skybox_id = s.allocate_id();
        s.entities.push(SceneEntityDef::new_empty(
            skybox_id,
            Some(env_id),
            "Skybox",
            EntityKind::Empty,
        ));

        let player_id = s.allocate_id();
        let mut player =
            SceneEntityDef::new_cube(player_id, Some(root_id), "Player", [0.0, 0.0, 0.0]);
        player.box_collider.as_mut().unwrap().team = TeamDef::Player;
        player.health = Some(HealthDef {
            current: 100,
            max: 100,
        });
        player.script = Some(ScriptDef {
            path: "scripts/Player.fy".into(),
            enabled: true,
        });
        s.entities.push(player);

        s
    }

    pub fn boss_demo() -> Self {
        let mut s = Self::sample_scene();
        s.name = "BossDemo".into();
        if let Some(root) = s.entities.first_mut() {
            root.name = "BossDemo".into();
        }
        let root_id = s.entities.first().map(|e| e.id);
        let boss_id = s.allocate_id();
        let mut boss = SceneEntityDef::new_model(
            boss_id,
            root_id,
            "FustyBoss",
            MeshKind::ObjModel,
            Some("models/crystal.obj".into()),
            [2.2, 0.0, 0.0],
            RgbaColor::new(205, 58, 86, 255),
        );
        boss.scale = [1.3, 1.3, 1.3];
        if let Some(mr) = &mut boss.mesh_renderer {
            mr.material_name = "BossCrimson_Mat".into();
        }
        if let Some(bc) = &mut boss.box_collider {
            bc.team = TeamDef::Enemy;
        }
        boss.health = Some(HealthDef {
            current: 1000,
            max: 1000,
        });
        boss.script = Some(ScriptDef {
            path: "examples/boss_arena.fy".into(),
            enabled: true,
        });
        s.entities.push(boss);

        let badge_id = s.allocate_id();
        let badge = SceneEntityDef::new_model(
            badge_id,
            root_id,
            "ArenaEmblem2D",
            MeshKind::Sprite2D,
            None,
            [-2.0, 0.3, -1.2],
            RgbaColor::new(255, 255, 255, 255),
        );
        s.entities.push(badge);
        s
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir '{}': {}", parent.display(), e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize scene error: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write '{}': {}", path.display(), e))
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
        serde_json::from_str(&json).map_err(|e| format!("Parse scene error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_sample_scene_matches_hierarchy_and_roundtrips() {
        let scene = SceneFile::sample_scene();
        let names: Vec<&str> = scene.entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "SampleScene",
                "MainCamera",
                "DirectionalLight",
                "Environment",
                "Ground",
                "Skybox",
                "Player"
            ]
        );

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("ferrium_sample_{}.json", nanos));
        scene.save_to_file(&tmp).unwrap();
        let loaded = SceneFile::load_from_file(&tmp).unwrap();
        assert_eq!(loaded.entities.len(), scene.entities.len());
        let _ = fs::remove_file(&tmp);
    }
}

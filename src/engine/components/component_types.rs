use serde::{Serialize, Deserialize};

/// Shared enum of all component types in the application
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Transform,
    Metadata,
    Camera,
    Collider,
    StaticObject3D,
    AnimatedObject3D,
    Shape,
    Material,
    Mesh,
    Animator,
    AnimationState,
    Skeleton,
}

impl ComponentType {
    /// Get the string representation of the component type
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentType::Transform => "Transform",
            ComponentType::Metadata => "Metadata",
            ComponentType::Camera => "Camera",
            ComponentType::Collider => "Collider",
            ComponentType::StaticObject3D => "StaticObject3D",
            ComponentType::AnimatedObject3D => "AnimatedObject3D",
            ComponentType::Shape => "Shape",
            ComponentType::Material => "Material",
            ComponentType::Mesh => "Mesh",
            ComponentType::Animator => "Animator",
            ComponentType::AnimationState => "AnimationState",
            ComponentType::Skeleton => "Skeleton",
        }
    }
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

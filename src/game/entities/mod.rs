pub mod testing_doll;
pub mod chair;
pub mod player;
pub mod blockout_platform;

#[allow(unused_imports)]
pub use testing_doll::spawn_testing_doll;
#[allow(unused_imports)]
pub use chair::spawn_chair;
pub use player::spawn_player;
pub use blockout_platform::spawn_blockout_platform;

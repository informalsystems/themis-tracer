/// Things that can be saved to disk
pub trait Save {
    fn save(&self) -> Result<(), String>;
}

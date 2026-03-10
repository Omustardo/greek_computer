use crate::menus::settings_menu::layout_menu::SavedLayout;
use std::collections::HashMap;

pub struct TripletNameGenerator;

impl TripletNameGenerator {
    const COLORS: &'static [&'static str] = &[
        "Red", "Blue", "Green", "Purple", "Orange", "Yellow", "Pink", "Cyan", "Magenta", "Lime", "Teal", "Navy",
        "Maroon", "Olive", "Silver", "Gold", "Coral", "Indigo", "Violet", "Crimson",
    ];

    const ANIMALS: &'static [&'static str] = &[
        "Tiger", "Eagle", "Dolphin", "Wolf", "Fox", "Bear", "Hawk", "Lion", "Shark", "Falcon", "Panther", "Raven",
        "Lynx", "Cobra", "Moose", "Otter", "Jaguar", "Whale", "Bison",
    ];

    const ADJECTIVES: &'static [&'static str] = &[
        "Swift", "Bold", "Calm", "Bright", "Sharp", "Strong", "Quick", "Wise", "Fierce", "Gentle", "Noble", "Silent",
        "Rapid", "Steady", "Clever", "Brave", "Agile", "Mighty", "Alert", "Graceful",
    ];

    pub fn generate_random() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        chrono::Local::now().hash(&mut hasher);
        let seed = hasher.finish();

        let color = Self::COLORS[(seed % Self::COLORS.len() as u64) as usize];
        let animal = Self::ANIMALS[((seed >> 8) % Self::ANIMALS.len() as u64) as usize];
        let adjective = Self::ADJECTIVES[((seed >> 16) % Self::ADJECTIVES.len() as u64) as usize];

        format!("{adjective} {color} {animal}")
    }

    pub fn generate_unique(existing_layouts: &HashMap<String, SavedLayout>) -> String {
        let mut attempts = 0;
        loop {
            let name = if attempts == 0 {
                Self::generate_random()
            } else {
                format!("{} ({})", Self::generate_random(), attempts)
            };

            if !existing_layouts.contains_key(&name) {
                return name;
            }

            attempts += 1;
            if attempts > 100 {
                return format!("Layout_{}", chrono::Local::now().timestamp());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_returns_non_empty() {
        let name = TripletNameGenerator::generate_random();
        assert!(!name.is_empty());
        assert_eq!(name.split_whitespace().count(), 3); // adjective color animal
    }

    #[test]
    fn test_generate_unique_no_conflicts() {
        let layouts = HashMap::new();
        let name = TripletNameGenerator::generate_unique(&layouts);
        assert!(!name.is_empty());
    }
}

use accesskit::Role as AccessKitRole;
use semantics::{Role, States};

pub(crate) fn accesskit_role(role: Role, states: &States) -> AccessKitRole {
    match role {
        Role::Window => AccessKitRole::Window,
        Role::Group => AccessKitRole::Group,
        Role::Text => AccessKitRole::Label,
        Role::Heading => AccessKitRole::Heading,
        Role::Button => AccessKitRole::Button,
        Role::Link => AccessKitRole::Link,
        Role::TextField | Role::SearchField if states.secure => AccessKitRole::PasswordInput,
        Role::TextField => AccessKitRole::TextInput,
        Role::SearchField => AccessKitRole::SearchInput,
        Role::Checkbox => AccessKitRole::CheckBox,
        Role::Switch => AccessKitRole::Switch,
        Role::RadioButton => AccessKitRole::RadioButton,
        Role::Slider => AccessKitRole::Slider,
        Role::ProgressBar => AccessKitRole::ProgressIndicator,
        Role::Image => AccessKitRole::Image,
        Role::List => AccessKitRole::List,
        Role::ListItem => AccessKitRole::ListItem,
        Role::ScrollView => AccessKitRole::ScrollView,
        Role::Menu => AccessKitRole::Menu,
        Role::MenuItem => AccessKitRole::MenuItem,
        Role::Tab => AccessKitRole::Tab,
        Role::TabList => AccessKitRole::TabList,
        Role::Dialog => AccessKitRole::Dialog,
        Role::Tooltip => AccessKitRole::Tooltip,
        Role::ComboBox => AccessKitRole::ComboBox,
        Role::Separator => AccessKitRole::Splitter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROLE_TABLE: [(Role, AccessKitRole); 25] = [
        (Role::Window, AccessKitRole::Window),
        (Role::Group, AccessKitRole::Group),
        (Role::Text, AccessKitRole::Label),
        (Role::Heading, AccessKitRole::Heading),
        (Role::Button, AccessKitRole::Button),
        (Role::Link, AccessKitRole::Link),
        (Role::TextField, AccessKitRole::TextInput),
        (Role::SearchField, AccessKitRole::SearchInput),
        (Role::Checkbox, AccessKitRole::CheckBox),
        (Role::Switch, AccessKitRole::Switch),
        (Role::RadioButton, AccessKitRole::RadioButton),
        (Role::Slider, AccessKitRole::Slider),
        (Role::ProgressBar, AccessKitRole::ProgressIndicator),
        (Role::Image, AccessKitRole::Image),
        (Role::List, AccessKitRole::List),
        (Role::ListItem, AccessKitRole::ListItem),
        (Role::ScrollView, AccessKitRole::ScrollView),
        (Role::Menu, AccessKitRole::Menu),
        (Role::MenuItem, AccessKitRole::MenuItem),
        (Role::Tab, AccessKitRole::Tab),
        (Role::TabList, AccessKitRole::TabList),
        (Role::Dialog, AccessKitRole::Dialog),
        (Role::Tooltip, AccessKitRole::Tooltip),
        (Role::ComboBox, AccessKitRole::ComboBox),
        (Role::Separator, AccessKitRole::Splitter),
    ];

    #[test]
    fn every_role_maps_to_its_accesskit_counterpart() {
        for (role, expected) in ROLE_TABLE {
            assert_eq!(accesskit_role(role, &States::default()), expected, "{role}");
        }
    }

    #[test]
    fn secure_text_entry_becomes_a_password_input() {
        let secure = States {
            secure: true,
            ..States::default()
        };
        assert_eq!(
            accesskit_role(Role::TextField, &secure),
            AccessKitRole::PasswordInput
        );
        assert_eq!(
            accesskit_role(Role::SearchField, &secure),
            AccessKitRole::PasswordInput
        );
        assert_eq!(accesskit_role(Role::Text, &secure), AccessKitRole::Label);
    }
}
